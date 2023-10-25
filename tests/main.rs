use {
    crate::utils::pda,
    hologram::{
        instructions::{CrateType, Faction},
        FUEL_ALLOWANCE_COOLDOWN,
    },
    instructions::utils::warp_forward,
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest, ProgramTestContext},
    solana_sdk::{signature::Keypair, signer::Signer},
    std::{str::FromStr, sync::Arc},
    tokio::sync::RwLock,
};

pub mod instructions;
pub mod utils;

const ADMIN: usize = 0;
const PAYER: usize = 1;
const USER_1: usize = 2;
const USER_2: usize = 3;
const USER_3: usize = 4;
const USER_4: usize = 5;
const USER_5: usize = 6;
const USER_6: usize = 7;

const REALM_NAME: &str = "HoloRealm";

const SWITCHBOARD_ATTESTATION_QUEUE: &str = "CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE";
const IMPERIUM_SSGF: &str = "5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe";
const IMPERIUM_AMF: &str = "4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL";
const IMPERIUM_CPF: &str = "EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG";

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_integration() {
    let mut program_test = ProgramTest::default();

    let keypairs: Vec<_> = std::iter::repeat_with(|| Arc::new(Keypair::new()))
        .take(8)
        .collect();

    // fund keypairs
    keypairs.iter().for_each(|user| {
        utils::create_and_fund_account(&user.pubkey(), &mut program_test);
    });

    // programs deployment
    {
        program_test.add_program("hologram", hologram::id(), processor!(hologram::entry));
    }

    // clone accounts from devnet
    {
        let devnet_accounts = vec![
            "SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f", // sbv2 programID
            "Fi8vncGpNKbq62gPo56G4toCehWNy77GgqGkTaAF5Lkk", // sbv2 IDL
            "CyZuD7RPDcrqCGbNvLCyqk6Py9cEZTKmNKujfPi3ynDd", // sbv2 SbState
            "sbattyXrzedoNATfc4L31wC9Mhxsi1BmFhTiN8gDshx", // sb attestation programID
            SWITCHBOARD_ATTESTATION_QUEUE,                 // sb attestation queue
            "5ExuoQR69trmKQfB95fDsUGsUrrChbGq9PFgt8qouncz", // sb devnet attestation IDL
            "5MFs7RGTjLi1wtKNBFRtuLipCkkjs4YQwRRU9sjnbQbS", // sb devnet programState
            IMPERIUM_SSGF,                                 // Imperium devnet ssgf
            IMPERIUM_AMF,                                  // Imperium devnet amf
            IMPERIUM_CPF,                                  // Imperium devnet cpf
        ]
        .into_iter()
        .map(|s| Pubkey::from_str(s).unwrap())
        .collect::<Vec<Pubkey>>();
        utils::clone_accounts_to_localnet(
            &mut program_test,
            &devnet_accounts,
            "https://api.devnet.solana.com".to_string(),
        )
        .await
        .unwrap();
    }

    // Start the client and connect to localnet validator
    let program_test_ctx: Arc<RwLock<ProgramTestContext>> =
        Arc::new(RwLock::new(program_test.start_with_context().await));

    let (realm_pda, _) = pda::get_realm_pda(&REALM_NAME.to_string());
    let ssgf = Pubkey::from_str(IMPERIUM_SSGF).unwrap();
    let amf = Pubkey::from_str(IMPERIUM_AMF).unwrap();
    let cpf = Pubkey::from_str(IMPERIUM_CPF).unwrap();

    // [1] --------------------------------- INITIALIZE REALM ------------------------------------
    {
        instructions::initialize_realm(
            &program_test_ctx,
            &keypairs[PAYER],
            &keypairs[ADMIN],
            &REALM_NAME.to_string(),
            &ssgf,
            &amf,
            &cpf,
        )
        .await
        .unwrap();
    }

    // [2] --------------------------------- CREATE USER ACCOUNT ---------------------------------
    {
        let mut create_user_account_tasks = vec![];
        [USER_1, USER_2, USER_3, USER_4, USER_5, USER_6]
            .iter()
            .for_each(|user| {
                let user = Arc::clone(&keypairs[*user]);
                let ctx = Arc::clone(&program_test_ctx);
                let task = tokio::spawn(async move {
                    instructions::create_user_account(&ctx, &user, &realm_pda)
                        .await
                        .unwrap();
                });
                create_user_account_tasks.push(task);
            });

        // Wait for all tasks to finish
        for task in create_user_account_tasks {
            task.await.unwrap();
        }
    }

    // [3] --------------------------------- CREATE SPACESHIP ------------------------------------
    {
        let names = [
            "HoloShip",
            "xXx_shadow_xXx",
            "El_Spashipo",
            "Ships",
            "Bambooship",
            "JeanMichelShip",
        ];
        let mut create_spaceships_tasks = vec![];
        [USER_1, USER_2, USER_3, USER_4, USER_5, USER_6]
            .iter()
            .enumerate()
            .for_each(|(i, user)| {
                let user = Arc::clone(&keypairs[*user]);
                let ctx = Arc::clone(&program_test_ctx);
                let admin_key = keypairs[ADMIN].pubkey();
                let task = tokio::spawn(async move {
                    instructions::create_spaceship(
                        &ctx,
                        &user,
                        &realm_pda,
                        &admin_key,
                        &names[i % names.len()].to_string(),
                    )
                    .await
                    .unwrap();
                });
                create_spaceships_tasks.push(task);
            });

        // Wait for all tasks to finish
        for task in create_spaceships_tasks {
            task.await.unwrap();
        }
    }

    // [4] ---------------------- CLAIM FUEL ALLOWANCE (should fail) ----------------------------------
    // This test should fail as the first claim is available 24h after the spaceship creation
    // ------------------------------------------------------------------------------------------------
    {
        let user = &keypairs[USER_1];
        // we pick the first spaceship of the player for this test
        let spaceship_index = 0;

        assert!(instructions::claim_fuel_allowance(
            &program_test_ctx,
            &user,
            &realm_pda,
            spaceship_index,
        )
        .await
        .is_err());
    }

    // [4 bis] ---------------------- CLAIM FUEL ALLOWANCE (should fail) ----------------------------
    // We now warp FUEL_ALLOWANCE_COOLDOWN seconds later
    warp_forward(&program_test_ctx, FUEL_ALLOWANCE_COOLDOWN + 1).await;
    // ---------------------------------------------------------------------------------------------
    {
        let user = &keypairs[USER_1];
        let spaceship_index = 0;

        instructions::claim_fuel_allowance(&program_test_ctx, &user, &realm_pda, spaceship_index)
            .await
            .unwrap();
    }

    // [5] -------------------- PICK CRATE ---------------------------------------------------------
    // Pick a crate for each spaceship (we vary the types of crates)
    // Only distribute NI crate as the players start with ImperialCredits only
    // ---------------------------------------------------------------------------------------------
    {
        let mut pick_crate_tasks = vec![];
        [USER_1, USER_2, USER_3, USER_4, USER_5, USER_6]
            .iter()
            .for_each(|user| {
                let user = Arc::clone(&keypairs[*user]);
                let ctx = Arc::clone(&program_test_ctx);

                let task = tokio::spawn(async move {
                    let spaceship_index = 0;
                    instructions::pick_crate(
                        &ctx,
                        &user,
                        &realm_pda,
                        spaceship_index,
                        CrateType::NavyIssue,
                    )
                    .await
                    .unwrap();
                });
                pick_crate_tasks.push(task);
            });
        // Wait for all tasks to finish
        for task in pick_crate_tasks {
            task.await.unwrap();
        }
    }

    // [6] -------------------- ARENA MATCHMAKING (queue filling) ----------------------------------
    // Start by placing 5 players in the queue
    // ---------------------------------------------------------------------------------------------
    {
        let mut arena_matchmaking_tasks = vec![];
        let factions = [Faction::Imperium, Faction::Pirate, Faction::RogueDrone];
        [USER_2, USER_3, USER_4, USER_5, USER_6]
            .iter()
            .enumerate()
            .for_each(|(i, user)| {
                let user = Arc::clone(&keypairs[*user]);
                let ctx = Arc::clone(&program_test_ctx);

                let task = async move {
                    let spaceship_index = 0;

                    instructions::arena_matchmaking(
                        &ctx,
                        &user,
                        &realm_pda,
                        spaceship_index,
                        factions[i % factions.len()],
                    )
                    .await
                    .unwrap();
                };
                arena_matchmaking_tasks.push(task);
            });

        // Here we want each task to be executed sequentially so that we can verify the results without interferences
        for task in arena_matchmaking_tasks {
            tokio::spawn(task).await.unwrap();
        }
    }

    // [7] ---------------------- ARENA MATCHMAKING (matching) -------------------------------------
    // Now that the queue is full, we can match the players
    // ---------------------------------------------------------------------------------------------
    // require to bypass validator protection to drop "similar IX" (we called the same in step [4])
    warp_forward(&program_test_ctx, 1).await;
    {
        let user = &keypairs[USER_1];
        let spaceship_index = 0;

        instructions::arena_matchmaking(
            &program_test_ctx,
            &user,
            &realm_pda,
            spaceship_index,
            Faction::Imperium,
        )
        .await
        .unwrap();
    }
}

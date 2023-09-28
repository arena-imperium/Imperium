use {
    hologram::state::UserAccount,
    solana_program::pubkey::Pubkey,
    solana_program_test::{processor, ProgramTest, ProgramTestContext},
    solana_sdk::{signature::Keypair, signer::Signer},
    std::{str::FromStr, sync::Arc},
    tokio::sync::RwLock,
};

pub mod instructions;
pub mod utils;

const UPGRADE_AUTHORITY: usize = 0;
const ADMIN: usize = 1;
const PAYER: usize = 2;
const USER_1: usize = 3;
const USER_2: usize = 4;
const USER_3: usize = 5;
const USER_4: usize = 6;
const USER_5: usize = 7;
const USER_6: usize = 8;

const REALM_NAME: &str = "HoloRealm";

const SWITCHBOARD_ATTESTATION_QUEUE: &str = "CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE";
const IMPERIUM_SSGF: &str = "CyxB4ZrDSL2jjgPs5nGP93UpfNPHN4X66Z26WhnaeEi5";
const IMPERIUM_AMF: &str = "HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog";
const IMPERIUM_CPF: &str = "EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG";

#[tokio::test]
pub async fn test_integration() {
    let mut program_test = ProgramTest::default();

    // create keypairs
    let keypairs = vec![Arc::new(Keypair::new()); 9];

    // fund users
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

    // Boostrap new queue
    // THE CODE WON"T work until we write the necessary
    // https://github.com/switchboard-xyz/solana-sdk/blob/main/javascript/solana.js/src/accounts/attestationQueueAccount.ts#L399
    // adapted to rust. Asking on discord.
    

    // [1] --------------------------------- INITIALIZE REALM ------------------------------------
    let ssgf = Pubkey::from_str(IMPERIUM_SSGF).unwrap();
    let amf = Pubkey::from_str(IMPERIUM_AMF).unwrap();
    let cpf = Pubkey::from_str("HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog").unwrap();

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

    // [2] --------------------------------- CREATE USERs ACCOUNT ---------------------------------
    instructions::create_user_account(
        &program_test_ctx,
        &keypairs[USER_1],
        &REALM_NAME.to_string(),
    )
    .await
    .unwrap();

    // utils::warp_forward(&program_test_ctx, 1000).await;

    // instructions::create_user_account(
    //     &program_test_ctx,
    //     &keypairs[USER_2],
    //     &REALM_NAME.to_string(),
    // )
    // .await
    // .unwrap();

    // utils::warp_forward(&program_test_ctx, 1000).await;

    // instructions::create_user_account(
    //     &program_test_ctx,
    //     &keypairs[USER_3],
    //     &REALM_NAME.to_string(),
    // )
    // .await
    // .unwrap();
    // instructions::create_user_account(
    //     &program_test_ctx,
    //     &keypairs[USER_4],
    //     &REALM_NAME.to_string(),
    // )
    // .await
    // .unwrap();
    // instructions::create_user_account(
    //     &program_test_ctx,
    //     &keypairs[USER_5],
    //     &REALM_NAME.to_string(),
    // )
    // .await
    // .unwrap();
    // instructions::create_user_account(
    //     &program_test_ctx,
    //     &keypairs[USER_6],
    //     &REALM_NAME.to_string(),
    // )
    // .await
    // .unwrap();
    // let mut create_user_account_tasks = vec![];
    // [USER_1, USER_2, USER_3, USER_4, USER_5, USER_6]
    //     .iter()
    //     .for_each(|user| {
    //         let keypair = Arc::clone(&keypairs[*user]);
    //         let realm_name = REALM_NAME.to_string();
    //         let ctx = Arc::clone(&program_test_ctx);
    //         let task = tokio::spawn(async move {
    //             instructions::create_user_account(&ctx, &keypair, &realm_name)
    //                 .await
    //                 .unwrap();
    //         });
    //         create_user_account_tasks.push(task);
    //     });

    // // Wait for all tasks to finish
    // for task in create_user_account_tasks {
    //     task.await.unwrap();
    // }

    // [3] --------------------------------- CREATE SPACESHIP ------------------------------------
    let spaceship_name = "HoloShip";
    instructions::create_spaceship(
        &program_test_ctx,
        &keypairs[USER_1],
        &keypairs[ADMIN].pubkey(),
        &REALM_NAME.to_string(),
        &spaceship_name.to_string(),
    )
    .await
    .unwrap();
}

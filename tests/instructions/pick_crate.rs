pub use crate::utils;
use {
    crate::{utils::pda, IMPERIUM_CPF, SWITCHBOARD_ATTESTATION_QUEUE},
    anchor_lang::ToAccountMetas,
    hologram::{
        instructions::CrateType,
        state::{SpaceShip, SwitchboardFunctionRequestStatus},
        utils::RandomNumberGenerator,
    },
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    spl_associated_token_account::get_associated_token_address,
    spl_token::native_mint,
    std::str::FromStr,
    tokio::sync::RwLock,
};

pub async fn pick_crate(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_pda: &Pubkey,
    realm_admin: &Pubkey,
    spaceship_pda: &Pubkey,
    crate_type: CrateType,
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let (user_account_pda, _) = pda::get_user_account_pda(&realm_pda, &user.pubkey());
    // Fetch the user account
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, spaceship_pda).await;
    let switchboard_cpf_request = spaceship.crate_picking.switchboard_request_info.account;
    let (switchboard_state_pda, _) = utils::get_switchboard_state_pda();
    let switchboard_cpf_request_escrow =
        get_associated_token_address(&switchboard_cpf_request, &native_mint::ID);

    let accounts_meta = {
        let accounts = hologram::accounts::PickCrate {
            user: user.pubkey(),
            admin: *realm_admin,
            realm: *realm_pda,
            user_account: user_account_pda,
            spaceship: *spaceship_pda,
            switchboard_state: switchboard_state_pda,
            switchboard_attestation_queue: Pubkey::from_str(SWITCHBOARD_ATTESTATION_QUEUE).unwrap(),
            crate_picking_function: Pubkey::from_str(IMPERIUM_CPF).unwrap(),
            switchboard_request: switchboard_cpf_request,
            switchboard_request_escrow: switchboard_cpf_request_escrow,
            system_program: solana_program::system_program::id(),
            token_program: switchboard_solana::anchor_spl::token::ID,
            switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::PickCrate { crate_type },
        Some(&user.pubkey()),
        &[user],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, spaceship_pda).await;

    // verify that the crate_picking status is updated
    assert!(matches!(
        spaceship.crate_picking.switchboard_request_info.status,
        SwitchboardFunctionRequestStatus::Requested { slot: _ }
    ));

    // ==== AND ===============================================================
    // Because using the localnet/banksclient setup we cannot rely on switchboard function, we call the settlement directly
    let spaceship_before = utils::get_account::<SpaceShip>(program_test_ctx, spaceship_pda).await;

    // ==== WHEN ==============================================================
    let enclave_signer = Keypair::new();
    let accounts_meta = {
        let accounts = hologram::accounts::PickCrateSettle {
            enclave_signer: enclave_signer.pubkey(), // In the real world this is not called by anyone else than the docker container
            user: user.pubkey(),
            realm: *realm_pda,
            user_account: user_account_pda,
            spaceship: *spaceship_pda,
            crate_picking_function: Pubkey::from_str(IMPERIUM_CPF).unwrap(),
            switchboard_request: switchboard_cpf_request,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    // The random generation is normally done on the switchboard side, but we do it here for the tests
    let generated_seed = rand::random::<u32>();

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::PickCrateSettle {
            crate_type,
            generated_seed,
        },
        Some(&user.pubkey()),
        &[&user, &enclave_signer],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    // verify that the request is settled
    assert!(matches!(
        spaceship.crate_picking.switchboard_request_info.status,
        SwitchboardFunctionRequestStatus::Settled { slot: _ }
    ));

    // verify that the crate cost was debited
    let price = crate_type.crate_price();
    let currency = crate_type.payment_currency();
    assert_eq!(
        spaceship_before.wallet.get_balance(currency) - price as u16,
        spaceship.wallet.get_balance(currency) as u16
    );

    // verify presence of drop (the RNG is deterministic)
    let mut rng = RandomNumberGenerator::new(generated_seed as u64);
    let crate_outcome_roll = rng.roll_dice(100) as u8;
    let crate_outcome = crate_type.determine_outcome(crate_outcome_roll);
    match crate_outcome {
        hologram::instructions::CrateOutcome::Module { .. } => {
            assert_eq!(spaceship.modules.len(), spaceship_before.modules.len() + 1);
            assert_eq!(spaceship.powerup_score, spaceship_before.powerup_score + 1);
        }
        hologram::instructions::CrateOutcome::Drone { .. } => {
            assert_eq!(spaceship.drones.len(), spaceship_before.drones.len() + 1);
            assert_eq!(spaceship.powerup_score, spaceship_before.powerup_score + 1);
        }
        hologram::instructions::CrateOutcome::Mutation => {
            assert_eq!(
                spaceship.mutations.len(),
                spaceship_before.mutations.len() + 1
            );
            assert_eq!(spaceship.powerup_score, spaceship_before.powerup_score + 1);
        }
        hologram::instructions::CrateOutcome::Scam => {
            assert_eq!(spaceship.modules.len(), spaceship_before.modules.len());
            assert_eq!(spaceship.drones.len(), spaceship_before.drones.len());
            assert_eq!(spaceship.mutations.len(), spaceship_before.mutations.len());
            assert_eq!(spaceship.powerup_score, spaceship_before.powerup_score);
        }
    }

    Ok(())
}

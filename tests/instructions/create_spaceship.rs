pub use crate::utils;
use {
    crate::{utils::pda, IMPERIUM_AMF, IMPERIUM_SSGF, SWITCHBOARD_ATTESTATION_QUEUE},
    anchor_lang::ToAccountMetas,
    hologram::{
        state::{SpaceShip, SwitchboardFunctionRequestStatus, UserAccount},
        BASE_MAX_FUEL,
    },
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    spl_associated_token_account::get_associated_token_address,
    spl_token::native_mint,
    std::str::FromStr,
    tokio::sync::RwLock,
};

pub async fn create_spaceship(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_admin: &Pubkey,
    realm_name: &String,
    spaceship_name: &String,
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let (realm_pda, _) = pda::get_realm_pda(realm_name);
    let (user_account_pda, _) = pda::get_user_account_pda(&realm_pda, &user.pubkey());

    // Fetch the user account
    let user_account = utils::get_account::<UserAccount>(program_test_ctx, user_account_pda).await;
    // Read the number of spaceships
    let num_spaceships = user_account.spaceships.len();

    let (spaceship_pda, spaceship_bump) =
        pda::get_spaceship_pda(&realm_pda, &user.pubkey(), num_spaceships);
    let (switchboard_state_pda, _) = utils::get_switchboard_state_pda();
    let switchboard_ssgf_request_keypair = Keypair::new();
    let switchboard_amf_request_keypair = Keypair::new();
    let switchboard_ssgf_request_escrow =
        get_associated_token_address(&switchboard_ssgf_request_keypair.pubkey(), &native_mint::ID);
    let switchboard_amf_request_escrow =
        get_associated_token_address(&switchboard_amf_request_keypair.pubkey(), &native_mint::ID);
    let user_wsol_token_account = get_associated_token_address(&user.pubkey(), &native_mint::ID);

    let accounts_meta = {
        let accounts = hologram::accounts::CreateSpaceship {
            user: user.pubkey(),
            realm: realm_pda,
            admin: *realm_admin,
            user_account: user_account_pda,
            spaceship: spaceship_pda,
            switchboard_state: switchboard_state_pda,
            switchboard_attestation_queue: Pubkey::from_str(SWITCHBOARD_ATTESTATION_QUEUE).unwrap(),
            spaceship_seed_generation_function: Pubkey::from_str(IMPERIUM_SSGF).unwrap(),
            switchboard_ssgf_request: switchboard_ssgf_request_keypair.pubkey(),
            switchboard_ssgf_request_escrow,
            arena_matchmaking_function: Pubkey::from_str(IMPERIUM_AMF).unwrap(),
            switchboard_amf_request: switchboard_amf_request_keypair.pubkey(),
            switchboard_amf_request_escrow,
            user_wsol_token_account,
            switchboard_mint: native_mint::ID,
            system_program: solana_program::system_program::id(),
            token_program: switchboard_solana::anchor_spl::token::ID,
            switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
            associated_token_program: switchboard_solana::anchor_spl::associated_token::ID,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::CreateSpaceship {
            name: spaceship_name.to_string(),
        },
        Some(&user.pubkey()),
        &[
            user,
            &switchboard_ssgf_request_keypair,
            &switchboard_amf_request_keypair,
        ],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, spaceship_pda).await;

    assert_eq!(spaceship.bump, spaceship_bump);
    assert_eq!(spaceship.owner, user.pubkey());
    assert_eq!(spaceship.name.to_string(), *spaceship_name);

    assert_eq!(
        spaceship.arena_matchmaking.switchboard_request_info.account,
        switchboard_amf_request_keypair.pubkey()
    );
    assert!(matches!(
        spaceship.arena_matchmaking.switchboard_request_info.status,
        SwitchboardFunctionRequestStatus::None
    ));

    assert_eq!(
        spaceship.randomness.switchboard_request_info.account,
        switchboard_ssgf_request_keypair.pubkey()
    );
    assert!(matches!(
        spaceship.randomness.switchboard_request_info.status,
        SwitchboardFunctionRequestStatus::Requested(_)
    ));

    assert_eq!(spaceship.fuel.max, BASE_MAX_FUEL);
    assert_eq!(spaceship.fuel.current, BASE_MAX_FUEL);
    Ok(())
}

pub use crate::utils;
use {
    crate::utils::pda,
    anchor_lang::{prelude::Pubkey, ToAccountMetas},
    hologram::{state::Realm, ARENA_MATCHMAKING_LEVEL_PER_RANGE, MAX_LEVEL},
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    tokio::sync::RwLock,
};

pub async fn initialize_realm(
    program_test_ctx: &RwLock<ProgramTestContext>,
    payer: &Keypair,
    admin: &Keypair,
    realm_name: &String,
    spaceship_seed_generation_function: &Pubkey,
    arena_matchmaking_function: &Pubkey,
    crate_picking_function: &Pubkey,
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let (realm_pda, realm_bump) = pda::get_realm_pda(realm_name);

    let accounts_meta = {
        let accounts = hologram::accounts::InitializeRealm {
            payer: payer.pubkey(),
            admin: admin.pubkey(),
            realm: realm_pda,
            spaceship_seed_generation_function: spaceship_seed_generation_function.clone(),
            arena_matchmaking_function: arena_matchmaking_function.clone(),
            crate_picking_function: crate_picking_function.clone(),
            system_program: anchor_lang::system_program::ID,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::InitializeRealm {
            name: realm_name.clone(),
        },
        Some(&payer.pubkey()),
        &[payer],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let realm_account = utils::get_account::<Realm>(program_test_ctx, realm_pda).await;

    assert_eq!(realm_account.bump, realm_bump);
    assert_eq!(realm_account.name.to_string(), *realm_name);
    assert_eq!(realm_account.admin, admin.pubkey());
    assert_eq!(realm_account.switchboard_info.authority, admin.pubkey());
    assert_eq!(
        realm_account
            .switchboard_info
            .spaceship_seed_generation_function,
        *spaceship_seed_generation_function
    );
    assert_eq!(
        realm_account.switchboard_info.arena_matchmaking_function,
        *arena_matchmaking_function
    );
    assert_eq!(
        realm_account.arena_matchmaking_queue.len(),
        (0..MAX_LEVEL)
            .step_by(ARENA_MATCHMAKING_LEVEL_PER_RANGE as usize)
            .len()
    );
    assert_eq!(realm_account.analytics.total_user_accounts, 0);
    assert_eq!(realm_account.analytics.total_spaceships_created, 0);
    assert_eq!(realm_account.analytics.total_arena_matches, 0);
    Ok(())
}

pub use crate::utils;
use {
    crate::utils::pda,
    anchor_lang::ToAccountMetas,
    hologram::{instructions::StatType, state::SpaceShip},
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    tokio::sync::RwLock,
};

pub async fn allocate_stat_point(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_pda: &Pubkey,
    spaceship_pda: &Pubkey,
    stat_type: StatType,
) -> std::result::Result<(), BanksClientError> {
    let spaceship_before = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    // ==== WHEN ==============================================================
    let (user_account_pda, _) = pda::get_user_account_pda(&realm_pda, &user.pubkey());

    let accounts_meta = {
        let accounts = hologram::accounts::AllocateStatPoint {
            user: user.pubkey(),
            realm: *realm_pda,
            user_account: user_account_pda,
            spaceship: *spaceship_pda,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::AllocateStatPoint { stat_type },
        Some(&user.pubkey()),
        &[user],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    assert_eq!(spaceship.experience.available_stat_points, false);
    match stat_type {
        StatType::ArmorLayering => assert_eq!(
            spaceship.stats.armor_layering,
            spaceship_before.stats.armor_layering + 1
        ),
        StatType::ShieldSubsystems => assert_eq!(
            spaceship.stats.shield_subsystems,
            spaceship_before.stats.shield_subsystems + 1
        ),
        StatType::TurretRigging => assert_eq!(
            spaceship.stats.turret_rigging,
            spaceship_before.stats.turret_rigging + 1
        ),
        StatType::ElectronicSubsystems => assert_eq!(
            spaceship.stats.electronic_subsystems,
            spaceship_before.stats.electronic_subsystems + 1
        ),
        StatType::Manoeuvering => assert_eq!(
            spaceship.stats.manoeuvering,
            spaceship_before.stats.manoeuvering + 1
        ),
    };

    Ok(())
}

pub use crate::utils;
use {
    crate::utils::pda,
    anchor_lang::ToAccountMetas,
    hologram::{instructions::Subsystem, state::SpaceShip},
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
    stat_type: Subsystem,
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

    // verify that we debited a stat point
    assert_eq!(
        spaceship.experience.available_subsystem_upgrade_points,
        spaceship_before
            .experience
            .available_subsystem_upgrade_points
            - 1
    );

    // verify that the stat was increased
    match stat_type {
        Subsystem::ArmorLayering => assert_eq!(
            spaceship.subsystems.armor_layering,
            spaceship_before.subsystems.armor_layering + 1
        ),
        Subsystem::ShieldSubsystems => assert_eq!(
            spaceship.subsystems.shield_subsystems,
            spaceship_before.subsystems.shield_subsystems + 1
        ),
        Subsystem::TurretRigging => assert_eq!(
            spaceship.subsystems.turret_rigging,
            spaceship_before.subsystems.turret_rigging + 1
        ),
        Subsystem::ElectronicSubsystems => assert_eq!(
            spaceship.subsystems.electronic_subsystems,
            spaceship_before.subsystems.electronic_subsystems + 1
        ),
        Subsystem::Manoeuvering => assert_eq!(
            spaceship.subsystems.manoeuvering,
            spaceship_before.subsystems.manoeuvering + 1
        ),
    };

    Ok(())
}

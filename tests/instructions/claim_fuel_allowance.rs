pub use crate::utils;
use {
    crate::utils::pda,
    anchor_lang::ToAccountMetas,
    hologram::{state::SpaceShip, FUEL_ALLOWANCE_AMOUNT},
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    std::cmp::min,
    tokio::sync::RwLock,
};

pub async fn claim_fuel_allowance(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_pda: &Pubkey,
    spaceship_pda: &Pubkey,
) -> std::result::Result<(), BanksClientError> {
    let spaceship_before = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    // ==== WHEN ==============================================================
    let (user_account_pda, _) = pda::get_user_account_pda(&realm_pda, &user.pubkey());

    let accounts_meta = {
        let accounts = hologram::accounts::ClaimFuelAllowance {
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
        hologram::instruction::ClaimFuelAllowance {},
        Some(&user.pubkey()),
        &[user],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    // verify that the fuel allowance was claimed
    assert!(
        spaceship.fuel.current
            == min(
                spaceship.fuel.max,
                spaceship_before.fuel.current + FUEL_ALLOWANCE_AMOUNT
            ),
    );

    // verify that the timestamp was updated
    assert!(
        spaceship.fuel.daily_allowance_last_collection
            > spaceship_before.fuel.daily_allowance_last_collection
    );

    Ok(())
}

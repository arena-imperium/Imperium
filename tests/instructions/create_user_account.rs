pub use crate::utils;
use {
    crate::utils::pda,
    anchor_lang::ToAccountMetas,
    hologram::state::UserAccount,
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    tokio::sync::RwLock,
};

pub async fn create_user_account(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_pda: &Pubkey,
) -> std::result::Result<(), BanksClientError> {
    // ==== WHEN ==============================================================
    let (user_account_pda, user_account_bump) =
        pda::get_user_account_pda(&realm_pda, &user.pubkey());

    let accounts_meta = {
        let accounts = hologram::accounts::CreateUserAccount {
            user: user.pubkey(),
            realm: *realm_pda,
            user_account: user_account_pda,
            system_program: anchor_lang::system_program::ID,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::CreateUserAccount {},
        Some(&user.pubkey()),
        &[user],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let user_account = utils::get_account::<UserAccount>(program_test_ctx, &user_account_pda).await;

    assert_eq!(user_account.bump, user_account_bump);
    assert_eq!(user_account.user, user.pubkey());
    assert_eq!(user_account.spaceships.len(), 0);

    Ok(())
}

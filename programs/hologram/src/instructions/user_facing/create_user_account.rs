use {
    crate::state::{Realm, UserAccount},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Account<'info, Realm>,

    #[account(
        init,
        payer=user,
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump,
        space = UserAccount::LEN
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct UserAccountCreated {
    pub realm_name: String,
    pub user: Pubkey,
    pub user_account: Pubkey,
}

pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
    // Initialize user account
    {
        let ua = &mut ctx.accounts.user_account;
        ua.bump = *ctx
            .bumps
            .get("user_account")
            .ok_or(ProgramError::InvalidSeeds)?;
        ua.user = *ctx.accounts.user.key;
        ua.spaceships = vec![];
    }

    // Update realm analytics
    {
        ctx.accounts.realm.analytics.total_user_accounts += 1;
    }

    emit!(UserAccountCreated {
        realm_name: ctx.accounts.realm.name.to_string(),
        user: ctx.accounts.user.key(),
        user_account: ctx.accounts.user_account.key(),
    });
    Ok(())
}

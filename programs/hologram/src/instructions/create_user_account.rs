use {
    crate::state::{Realm, UserAccount},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer=user,
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump,
        space = UserAccount::LEN
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes().as_ref()],
        bump = realm.bump,
    )]
    pub realm: Account<'info, Realm>,

    pub system_program: Program<'info, System>,
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

    // Update realm stats
    {
        ctx.accounts.realm.stats.total_user_accounts += 1;
    }
    Ok(())
}

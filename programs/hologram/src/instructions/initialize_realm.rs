use {
    crate::{error::HologramError, state::Realm, utils::LimitedString},
    anchor_lang::prelude::*,
    switchboard_solana::FunctionAccountData,
};
// The realm represent the game world. It is the top level of the game hierarchy.

#[derive(Accounts)]
#[instruction(name:String)]
pub struct InitializeRealm<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: any
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer=payer,
        seeds=[b"realm", name.as_bytes()],
        bump,
        space = Realm::LEN,
    )]
    pub realm: Account<'info, Realm>,

    // Randomness generator function from Switchboard (custom)
    #[account(
        constraint =
            // Ensure our authority owns this function
            switchboard_function.load()?.authority == *admin.key &&
            // Ensure custom requests are allowed
            !switchboard_function.load()?.requests_disabled
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_realm(ctx: Context<InitializeRealm>, name: String) -> Result<()> {
    // Checks
    {
        // verify input parameters
        require!(
            name.len() <= LimitedString::MAX_LENGTH,
            HologramError::LimitedStringLengthExceeded
        );
    }

    // Initialize Realm account
    {
        ctx.accounts.realm.bump = *ctx.bumps.get("realm").ok_or(ProgramError::InvalidSeeds)?;
        ctx.accounts.realm.name = LimitedString::new(name);
        ctx.accounts.realm.admin = ctx.accounts.admin.key();
        ctx.accounts.realm.randomness.function = ctx.accounts.switchboard_function.key();
        ctx.accounts.realm.randomness.authority = ctx.accounts.admin.key();
    }

    Ok(())
}

use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, UserAccount},
    },
    anchor_lang::prelude::*,
    switchboard_solana::prelude::*,
};

#[derive(Accounts)]
pub struct ClaimFuelAllowance<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Box<Account<'info, Realm>>,

    #[account(
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        bump = spaceship.bump,
    )]
    pub spaceship: Account<'info, SpaceShip>,
}

pub fn claim_fuel_allowance(ctx: Context<ClaimFuelAllowance>) -> Result<()> {
    let current_time = Realm::get_time()?;

    ctx.accounts.spaceship.claim_fuel_allowance(current_time)?;

    Ok(())
}

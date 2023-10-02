use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, UserAccount},
    },
    anchor_lang::prelude::*,
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
        // seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        // bump = spaceship.bump,
        constraint = user_account.spaceships.iter().map(|s|{s.spaceship}).collect::<Vec<_>>().contains(&spaceship.key()),
    )]
    pub spaceship: Account<'info, SpaceShip>,
}

#[event]
pub struct FuelAllowanceClaimed {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: Pubkey,
    pub claim_timestamp: i64,
}

pub fn claim_fuel_allowance(ctx: Context<ClaimFuelAllowance>) -> Result<()> {
    let current_time = Realm::get_time()?;

    // validations
    {
        // check that the fuel allowance can be claimed (using timestamp)
        require!(
            ctx.accounts
                .spaceship
                .fuel_allowance_is_available(current_time)?,
            HologramError::FuelAllowanceOnCooldown
        );
    }

    // give allowance
    {
        ctx.accounts.spaceship.claim_fuel_allowance(current_time)?;
    }

    // emit event
    emit!(FuelAllowanceClaimed {
        realm_name: ctx.accounts.realm.name.to_string(),
        user: ctx.accounts.user.key(),
        spaceship: ctx.accounts.spaceship.key(),
        claim_timestamp: current_time,
    });

    Ok(())
}

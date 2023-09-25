use {
    crate::{
        error::HologramError,
        state::{Fuel, Realm, SpaceShip, UserAccount},
        DAILY_FUEL_ALLOWANCE, FUEL_ALLOWANCE_COOLDOWN,
    },
    anchor_lang::prelude::*,
    std::cmp::min,
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

impl SpaceShip {
    pub fn claim_fuel_allowance(&mut self, current_time: i64) -> Result<()> {
        require!(
            self.fuel.fuel_allowance_is_available(current_time)?,
            HologramError::FuelAllowanceOnCooldown
        );
        self.fuel.current = min(
            self.fuel.max,
            self.fuel
                .current
                .checked_add(DAILY_FUEL_ALLOWANCE)
                .ok_or(HologramError::Overflow)?,
        );
        self.fuel.daily_allowance_last_collection = current_time;
        Ok(())
    }
}

impl Fuel {
    fn fuel_allowance_is_available(&self, current_time: i64) -> Result<bool> {
        let cooldown = current_time
            .checked_sub(FUEL_ALLOWANCE_COOLDOWN)
            .ok_or(HologramError::Overflow)?;
        Ok(self.daily_allowance_last_collection < cooldown)
    }
}

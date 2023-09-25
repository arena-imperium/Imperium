use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, StatsType, UserAccount},
    },
    anchor_lang::prelude::*,
    switchboard_solana::prelude::*,
};

#[derive(Accounts)]
pub struct AllocateStatPoint<'info> {
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

pub fn allocate_stat_point(ctx: Context<AllocateStatPoint>, stat_type: StatsType) -> Result<()> {
    ctx.accounts.spaceship.increase_stat(stat_type)?;
    Ok(())
}

impl SpaceShip {
    pub fn increase_stat(&mut self, stat_type: StatsType) -> Result<()> {
        require!(
            self.experience.available_stats_points,
            HologramError::NoAvailableStatsPoints
        );
        match stat_type {
            StatsType::ArmorLayering => self.stats.armor_layering += 1,
            StatsType::ShieldSubsystems => self.stats.shield_subsystems += 1,
            StatsType::TurretRigging => self.stats.turret_rigging += 1,
            StatsType::ElectronicSubsystems => self.stats.electronic_subsystems += 1,
            StatsType::Manoeuvering => self.stats.manoeuvering += 1,
        }
        self.experience.available_stats_points = false;
        Ok(())
    }
}

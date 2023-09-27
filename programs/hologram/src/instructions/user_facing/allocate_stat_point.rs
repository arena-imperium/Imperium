use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, UserAccount},
    },
    anchor_lang::prelude::*,
    switchboard_solana::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum StatType {
    ArmorLayering,
    ShieldSubsystems,
    TurretRigging,
    ElectronicSubsystems,
    Manoeuvering,
}

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

pub fn allocate_stat_point(ctx: Context<AllocateStatPoint>, stat_type: StatType) -> Result<()> {
    ctx.accounts.spaceship.increase_stat(stat_type)?;
    Ok(())
}

impl SpaceShip {
    pub fn increase_stat(&mut self, stat_type: StatType) -> Result<()> {
        require!(
            self.experience.available_stats_points,
            HologramError::NoAvailableStatsPoints
        );
        match stat_type {
            StatType::ArmorLayering => self.stats.armor_layering += 1,
            StatType::ShieldSubsystems => self.stats.shield_subsystems += 1,
            StatType::TurretRigging => self.stats.turret_rigging += 1,
            StatType::ElectronicSubsystems => self.stats.electronic_subsystems += 1,
            StatType::Manoeuvering => self.stats.manoeuvering += 1,
        }
        self.experience.available_stats_points = false;
        Ok(())
    }
}

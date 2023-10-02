use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, SpaceShipLite, UserAccount},
    },
    anchor_lang::prelude::*,
    std::borrow::BorrowMut,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
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
        // seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        // bump = spaceship.bump,
        constraint = user_account.spaceships.iter().map(|s|{s.spaceship}).collect::<Vec<_>>().contains(&spaceship.key()),
    )]
    pub spaceship: Account<'info, SpaceShip>,
}

#[event]
pub struct StatPointAllocated {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
    pub stat_type: StatType,
}

pub fn allocate_stat_point(ctx: Context<AllocateStatPoint>, stat_type: StatType) -> Result<()> {
    // validation
    {
        // spaceship must have an available stat point to spend
        require!(
            ctx.accounts.spaceship.experience.available_stat_points > 0,
            HologramError::NoAvailableStatsPoints
        );
    }

    // allocate stat point
    {
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        match stat_type {
            StatType::ArmorLayering => spaceship.stats.armor_layering += 1,
            StatType::ShieldSubsystems => spaceship.stats.shield_subsystems += 1,
            StatType::TurretRigging => spaceship.stats.turret_rigging += 1,
            StatType::ElectronicSubsystems => spaceship.stats.electronic_subsystems += 1,
            StatType::Manoeuvering => spaceship.stats.manoeuvering += 1,
        }
    }

    // spend the stat point
    {
        ctx.accounts.spaceship.experience.debit_stat_point(1)?;
    }
    emit!(StatPointAllocated {
        realm_name: String::from(ctx.accounts.realm.name),
        user: *ctx.accounts.user.key,
        spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship),
        stat_type
    });
    Ok(())
}

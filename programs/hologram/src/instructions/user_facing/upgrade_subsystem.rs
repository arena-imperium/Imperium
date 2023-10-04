use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, SpaceShipLite, UserAccount},
        MAX_HULL_INTEGRITY_SUBSYSTEM_LEVEL, MAX_SHIELD_SUBSYSTEM_LEVEL,
    },
    anchor_lang::prelude::*,
    std::borrow::BorrowMut,
};

// Equivalent to Stats in an RPG
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum Subsystem {
    Shield,
    HullIntegrity,
    WeaponRigging,
    Manoeuvering,
}

#[derive(Accounts)]
pub struct UpgradeSubsystem<'info> {
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
pub struct SubsystemUpgraded {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
    pub subsystem: Subsystem,
}

pub fn upgrade_subsystem(ctx: Context<UpgradeSubsystem>, subsystem: Subsystem) -> Result<()> {
    // validation
    {
        // spaceship must have an available upgrade point to spend
        require!(
            ctx.accounts
                .spaceship
                .experience
                .available_subsystem_upgrade_points
                > 0,
            HologramError::NoAvailableSubsystemUpgradePoint
        );
    }

    // upgrade subsystem
    {
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        match subsystem {
            Subsystem::Manoeuvering => spaceship.subsystems.manoeuvering += 1,
            Subsystem::Shield => {
                require!(
                    spaceship.subsystems.shield < MAX_SHIELD_SUBSYSTEM_LEVEL,
                    HologramError::MaxSubsystemLevelReached
                );
                spaceship.subsystems.shield += 1;
            }
            Subsystem::HullIntegrity => {
                require!(
                    spaceship.subsystems.hull_integrity < MAX_HULL_INTEGRITY_SUBSYSTEM_LEVEL,
                    HologramError::MaxSubsystemLevelReached
                );
                spaceship.subsystems.hull_integrity += 1;
            }
            Subsystem::WeaponRigging => spaceship.subsystems.weapon_rigging += 1,
        }
    }

    // spend the upgrade point
    {
        ctx.accounts
            .spaceship
            .experience
            .debit_subsystem_upgrade_point(1)?;
    }
    emit!(SubsystemUpgraded {
        realm_name: String::from(ctx.accounts.realm.name),
        user: *ctx.accounts.user.key,
        spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship),
        subsystem
    });
    Ok(())
}

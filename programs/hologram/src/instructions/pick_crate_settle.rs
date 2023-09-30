use {
    anchor_lang::prelude::*,
    crate::{
        engine::LootEngine,
        error::HologramError,
        state::{
            spaceship::{self},
            Realm, SpaceShip, UserAccount,
        },
        utils::RandomNumberGenerator,
    },
    spaceship::SwitchboardFunctionRequestStatus,
    std::borrow::BorrowMut,
    switchboard_solana::FunctionAccountData,
};

#[allow(unused_imports)]
use switchboard_solana::FunctionRequestAccountData;

// total of each category must be 100 (%)

pub const NIS_MODULE_CHANCE: u8 = 80;
pub const NIS_DRONE_CHANCE: u8 = 20;
pub const NIS_MUTATION_CHANCE: u8 = 0;
pub const NIS_SCAM_CHANCE: u8 = 0;
pub const NIS_FACTION_RARITY_ENABLED: bool = false;
pub const NIS_EXOTIC_MODULE_ENABLED: bool = false;

pub const PC_MODULE_CHANCE: u8 = 45;
pub const PC_DRONE_CHANCE: u8 = 42;
pub const PC_MUTATION_CHANCE: u8 = 5;
pub const PC_SCAM_CHANCE: u8 = 8;
pub const PC_FACTION_RARITY_ENABLED: bool = true;
pub const PC_EXOTIC_MODULE_ENABLED: bool = false;

pub const AAC_MODULE_CHANCE: u8 = 40;
pub const AAC_DRONE_CHANCE: u8 = 15;
pub const AAC_MUTATION_CHANCE: u8 = 40;
pub const AAC_SCAM_CHANCE: u8 = 5;
pub const AAC_FACTION_RARITY_ENABLED: bool = false;
pub const AAC_EXOTIC_MODULE_ENABLED: bool = true;

#[derive(Accounts)]
pub struct PickCrateSettle<'info> {
    /// CHECK: verified in the arena_matchmaking_function (to make sure it was called by the container)
    #[account()]
    pub enclave_signer: Signer<'info>,

    #[account(mut)]
    /// CHECK: forwarded from the create_spaceship IX (and validated by it)
    pub user: AccountInfo<'info>,

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

    // Note: The spaceship is pre-resized in pick_crate, where we always realloc a space for each type of power-up preemptively
    //  if we were to do so here, it would complicate things with the SBf() payer
    #[account(
        mut,
        // seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        // bump = spaceship.bump,
        constraint = user_account.spaceships.iter().map(|s|{s.spaceship}).collect::<Vec<_>>().contains(&spaceship.key()),
    )]
    pub spaceship: Account<'info, SpaceShip>,

    #[account( 
        // validate that we use the realm custom switchboard function
        constraint = realm.switchboard_info.crate_picking_function == crate_picking_function.key(),
    )]
    pub crate_picking_function: AccountLoader<'info, FunctionAccountData>,

    #[cfg(not(any(test, feature = "testing")))]
    #[account(
        // validation of the signer is done in the IX code
    )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
    #[cfg(any(test, feature = "testing"))]
    /// CHECK: test target only
    pub switchboard_request: AccountInfo<'info>,
}

pub fn pick_crate_settle(
    ctx: Context<PickCrateSettle>,
    generated_seed: u32,
    crate_type: CrateType,
) -> Result<()> {
    // Validations
    {
        // verify that the call was made by the container
        // Disabled during tests
        #[cfg(not(any(test, feature = "testing")))]
        require!(
            ctx.accounts.switchboard_request.validate_signer(&ctx.accounts.crate_picking_function.to_account_info(), &ctx.accounts.enclave_signer.to_account_info()) == Ok(true),
            HologramError::FunctionValidationFailed
        );

        // verify that the request is pending settlement
        require!(
            ctx.accounts
                .spaceship
                .crate_picking
                .switchboard_request_info
                .is_requested(),
            HologramError::CratePickingAlreadySettled
        );

        // // verify that the switchboard request was successful
        // #[cfg(not(any(test, feature = "testing")))]
        // require!(
        //     ctx.accounts.switchboard_request.active_request.status == RequestStatus::RequestSuccess,
        //     HologramError::SwitchboardRequestNotSuccessful
        // );
    }
    let mut rng = RandomNumberGenerator::new(generated_seed as u64);
    let crate_outcome_roll = rng.roll_dice(100) as u8;

    // depending of player crate choice, allocate a module, a drone, a mutation or... nothing based on RNG
    {
        let crate_outcome = crate_type.determine_outcome(crate_outcome_roll);
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        match crate_outcome {
            CrateOutcome::Module {
                faction_rarity_enabled,
                exotic_module_enabled,
            } => {
                let module =
                    LootEngine::drop_module(&mut rng, faction_rarity_enabled, exotic_module_enabled)?;
                msg!("Module dropped: {:?}", module);
                spaceship.modules.push(module);
            }
            CrateOutcome::Drone {
                faction_rarity_enabled,
            } => {
                let drone = LootEngine::drop_drone(&mut rng, faction_rarity_enabled)?;
                msg!("Drone dropped: {:?}", drone);
                spaceship.drones.push(drone);
            }
            CrateOutcome::Mutation => {
                let mutation = LootEngine::drop_mutation(&mut rng, &spaceship.mutations)?;
                msg!("Mutation dropped: {:?}", mutation);
                spaceship.mutations.push(mutation);
            }
            CrateOutcome::Scam => {
                // no op
            }
        }

        // consume spaceship crate drop
        {
            ctx.accounts.spaceship.experience.available_crate = false;
        }

        // update spaceship crate_picking request
        {
            ctx.accounts
                .spaceship
                .crate_picking
                .switchboard_request_info
                .status = SwitchboardFunctionRequestStatus::Settled {
                slot: Realm::get_slot()?,
            };
        }
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum CrateType {
    NavyIssue,
    PirateContraband,
    BlackMarket,
}

#[derive(Clone, Copy)]
pub enum CrateOutcome {
    Module {
        faction_rarity_enabled: bool,
        exotic_module_enabled: bool,
    },
    Drone {
        faction_rarity_enabled: bool,
    },
    Mutation,
    Scam, // no drop
}

impl CrateType {
    // Determine the outcome of the crate based on the roll
    pub fn determine_outcome(&self, roll: u8) -> CrateOutcome {
        let crate_chances = match self {
            CrateType::NavyIssue => [
                (NIS_MODULE_CHANCE, CrateOutcome::Module {
                    faction_rarity_enabled: NIS_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: NIS_EXOTIC_MODULE_ENABLED,
                }),
                (NIS_DRONE_CHANCE, CrateOutcome::Drone {
                    faction_rarity_enabled: NIS_FACTION_RARITY_ENABLED,
                }),
                (NIS_MUTATION_CHANCE, CrateOutcome::Mutation),
                (NIS_SCAM_CHANCE, CrateOutcome::Scam),
            ],
            CrateType::PirateContraband => [
                (PC_MODULE_CHANCE, CrateOutcome::Module {
                    faction_rarity_enabled: PC_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: PC_EXOTIC_MODULE_ENABLED,
                }),
                (PC_DRONE_CHANCE, CrateOutcome::Drone {
                    faction_rarity_enabled: PC_FACTION_RARITY_ENABLED,
                }),
                (PC_MUTATION_CHANCE, CrateOutcome::Mutation),
                (PC_SCAM_CHANCE, CrateOutcome::Scam),
            ],
            CrateType::BlackMarket => [
                (AAC_MODULE_CHANCE, CrateOutcome::Module {
                    faction_rarity_enabled: AAC_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: AAC_EXOTIC_MODULE_ENABLED,
                }),
                (AAC_DRONE_CHANCE, CrateOutcome::Drone {
                    faction_rarity_enabled: AAC_FACTION_RARITY_ENABLED,
                }),
                (AAC_MUTATION_CHANCE, CrateOutcome::Mutation),
                (AAC_SCAM_CHANCE, CrateOutcome::Scam),
            ],
        };

        let mut cumulative_chance = 0;
        for (chance, outcome) in crate_chances.iter() {
            cumulative_chance += chance;
            if roll <= cumulative_chance {
                return *outcome;
            }
        }

        panic!("Invalid dice roll")
    }
}

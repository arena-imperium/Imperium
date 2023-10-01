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

use spaceship::Currency;
#[allow(unused_imports)]
use switchboard_solana::FunctionRequestAccountData;

// total of each category must be 100 (%)

pub const NI_CURRENCY: Currency = Currency::ImperialCredit;
pub const NI_PRICE: u8 = 25;
//s
pub const NI_MODULE_CHANCE: u8 = 80;
pub const NI_DRONE_CHANCE: u8 = 20;
pub const NI_MUTATION_CHANCE: u8 = 0;
pub const NI_SCAM_CHANCE: u8 = 0;
pub const NI_FACTION_RARITY_ENABLED: bool = false;
pub const NI_EXOTIC_MODULE_ENABLED: bool = false;

pub const PC_CURRENCY: Currency = Currency::ImperialCredit;
pub const PC_PRICE: u8 = 30;
//
pub const PC_MODULE_CHANCE: u8 = 45;
pub const PC_DRONE_CHANCE: u8 = 42;
pub const PC_MUTATION_CHANCE: u8 = 5;
pub const PC_SCAM_CHANCE: u8 = 8;
pub const PC_FACTION_RARITY_ENABLED: bool = true;
pub const PC_EXOTIC_MODULE_ENABLED: bool = false;

pub const BMC_CURRENCY: Currency = Currency::ActivateNanitePaste;
pub const BMC_PRICE: u8 = 35;
//
pub const BMC_MODULE_CHANCE: u8 = 40;
pub const BMC_DRONE_CHANCE: u8 = 15;
pub const BMC_MUTATION_CHANCE: u8 = 40;
pub const BMC_SCAM_CHANCE: u8 = 5;
pub const BMC_FACTION_RARITY_ENABLED: bool = false;
pub const BMC_EXOTIC_MODULE_ENABLED: bool = true;

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

    // purchase the crate (built-in balance validation)
    {
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        spaceship.wallet.debit(crate_type.crate_price() as u16, crate_type.payment_currency())?;
    }

    // depending of player crate choice, allocate a module, a drone, a mutation or... nothing based on RNG
    {
        let mut rng = RandomNumberGenerator::new(generated_seed as u64);
        let crate_outcome_roll = rng.roll_dice(100) as u8;

        let crate_outcome = crate_type.determine_outcome(crate_outcome_roll);
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        match crate_outcome {
            CrateOutcome::Module {
                faction_rarity_enabled,
                exotic_module_enabled,
            } => {
                let module =
                    LootEngine::drop_module(&mut rng, faction_rarity_enabled, exotic_module_enabled)?;
                spaceship.mount_module(module)?;
            }
            CrateOutcome::Drone {
                faction_rarity_enabled,
            } => {
                let drone = LootEngine::drop_drone(&mut rng, faction_rarity_enabled)?;
                spaceship.load_drone(drone)?;
            }
            CrateOutcome::Mutation => {
                let mutation = LootEngine::drop_mutation(&mut rng, &spaceship.mutations)?;
                spaceship.apply_mutation(mutation)?;
            }
            CrateOutcome::Scam => {
                 msg!("You've been scammed...");
                // no op
            }
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
    BiomechanicalCache,
}

impl CrateType {
    pub fn payment_currency(&self) -> Currency {
        match self {
            CrateType::NavyIssue => Currency::ImperialCredit,
            CrateType::PirateContraband => Currency::ImperialCredit,
            CrateType::BiomechanicalCache => Currency::ActivateNanitePaste,
        }
    }

    pub fn crate_price(&self) -> u8 {
        match self {
            CrateType::NavyIssue => NI_PRICE,
            CrateType::PirateContraband => PC_PRICE,
            CrateType::BiomechanicalCache => BMC_PRICE,
        }
    }
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
                (NI_MODULE_CHANCE, CrateOutcome::Module {
                    faction_rarity_enabled: NI_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: NI_EXOTIC_MODULE_ENABLED,
                }),
                (NI_DRONE_CHANCE, CrateOutcome::Drone {
                    faction_rarity_enabled: NI_FACTION_RARITY_ENABLED,
                }),
                (NI_MUTATION_CHANCE, CrateOutcome::Mutation),
                (NI_SCAM_CHANCE, CrateOutcome::Scam),
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
            CrateType::BiomechanicalCache => [
                (BMC_MODULE_CHANCE, CrateOutcome::Module {
                    faction_rarity_enabled: BMC_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: BMC_EXOTIC_MODULE_ENABLED,
                }),
                (BMC_DRONE_CHANCE, CrateOutcome::Drone {
                    faction_rarity_enabled: BMC_FACTION_RARITY_ENABLED,
                }),
                (BMC_MUTATION_CHANCE, CrateOutcome::Mutation),
                (BMC_SCAM_CHANCE, CrateOutcome::Scam),
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

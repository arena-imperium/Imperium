use {
    crate::{
        error::HologramError,
        state::{
            spaceship::{self, Module},
            Realm, SpaceShip, UserAccount,
        },
        utils::{LimitedString, RandomNumberGenerator},
    },
    anchor_lang::prelude::*,
    spaceship::{
        DamageProfile, Drone, ModuleClass, Mutation, Rarity, SwitchboardFunctionRequestStatus,
        WeaponClass, WeaponModuleStats,
    },
    std::borrow::BorrowMut,
    switchboard_solana::prelude::*,
};

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

pub fn pick_crate_settle(
    ctx: Context<PickCrateSettle>,
    generated_seed: u32,
    crate_type: CrateType,
) -> Result<()> {
    // Validations
    {
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
        let mut additional_spaceship_data_len = 0;
        let spaceship = ctx.accounts.spaceship.borrow_mut();
        match crate_outcome {
            CrateOutcome::Module {
                faction_rarity_enabled,
                exotic_module_enabled,
            } => {
                let module =
                    Realm::drop_module(&mut rng, faction_rarity_enabled, exotic_module_enabled)?;
                spaceship.modules.push(module);
                additional_spaceship_data_len = std::mem::size_of::<Module>();
            }
            CrateOutcome::Drone {
                faction_rarity_enabled,
            } => {
                let drone = Realm::drop_drone(&mut rng, faction_rarity_enabled)?;
                spaceship.drones.push(drone);
                additional_spaceship_data_len = std::mem::size_of::<Drone>();
            }
            CrateOutcome::Mutation => {
                let mutation = Realm::drop_mutation(&mut rng, &spaceship.mutations)?;
                spaceship.mutations.push(mutation);
                additional_spaceship_data_len = std::mem::size_of::<Mutation>();
            }
            CrateOutcome::Scam => {
                // no op
            }
        }

        // resize spaceship account based on the drop
        {
            let spaceship_ai = ctx.accounts.spaceship.to_account_info();
            let current_data_len = spaceship_ai.data_len();
            let new_data_len = current_data_len + additional_spaceship_data_len;
            spaceship_ai.realloc(new_data_len, false)?;
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

impl Realm {
    pub fn drop_module(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
        exotic_weapon_enabled: bool,
    ) -> Result<Module> {
        let roll = rng.roll_dice(100);

        Ok(Module {
            name: LimitedString::new("150mm Light Autocannon"),
            rarity: Rarity::Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: WeaponClass::Projectile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 0,
                    kinetic: 2,
                    explosive: 0,
                },
                charge_time: 8,
                projectile_speed: 50,
                shots: 4,
            }),
        })
    }

    pub fn drop_drone(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
    ) -> Result<Drone> {
        let roll = rng.roll_dice(100);

        Ok(Drone {
            name: LimitedString::new("Warrior II"),
            rarity: Rarity::Uncommon,
        })
    }

    pub fn drop_mutation(
        rng: &mut RandomNumberGenerator,
        owned_mutation: &Vec<Mutation>,
    ) -> Result<Mutation> {
        let roll = rng.roll_dice(100);

        Ok(Mutation {
            name: LimitedString::new("Fungal Growth"),
            rarity: Rarity::Common,
        })
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum CrateType {
    NavyIssue,
    PirateContraband,
    BlackMarket,
}

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
        match self {
            CrateType::NavyIssue => match roll {
                r if r <= NIS_MODULE_CHANCE => CrateOutcome::Module {
                    faction_rarity_enabled: NIS_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: NIS_EXOTIC_MODULE_ENABLED,
                },
                r if r <= NIS_MODULE_CHANCE + NIS_DRONE_CHANCE => CrateOutcome::Drone {
                    faction_rarity_enabled: NIS_FACTION_RARITY_ENABLED,
                },
                r if r <= NIS_MODULE_CHANCE + NIS_DRONE_CHANCE + NIS_MUTATION_CHANCE => {
                    CrateOutcome::Mutation
                }
                _ => CrateOutcome::Scam,
            },
            CrateType::PirateContraband => match roll {
                r if r <= PC_MODULE_CHANCE => CrateOutcome::Module {
                    faction_rarity_enabled: PC_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: PC_EXOTIC_MODULE_ENABLED,
                },
                r if r <= PC_MODULE_CHANCE + PC_DRONE_CHANCE => CrateOutcome::Drone {
                    faction_rarity_enabled: PC_FACTION_RARITY_ENABLED,
                },
                r if r <= PC_MODULE_CHANCE + PC_DRONE_CHANCE + PC_MUTATION_CHANCE => {
                    CrateOutcome::Mutation
                }
                _ => CrateOutcome::Scam,
            },
            CrateType::BlackMarket => match roll {
                r if r <= AAC_MODULE_CHANCE => CrateOutcome::Module {
                    faction_rarity_enabled: AAC_FACTION_RARITY_ENABLED,
                    exotic_module_enabled: AAC_EXOTIC_MODULE_ENABLED,
                },
                r if r <= AAC_MODULE_CHANCE + AAC_DRONE_CHANCE => CrateOutcome::Drone {
                    faction_rarity_enabled: AAC_FACTION_RARITY_ENABLED,
                },
                r if r <= AAC_MODULE_CHANCE + AAC_DRONE_CHANCE + AAC_MUTATION_CHANCE => {
                    CrateOutcome::Mutation
                }
                _ => CrateOutcome::Scam,
            },
        }
    }
}

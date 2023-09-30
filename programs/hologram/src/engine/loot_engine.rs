use {
    crate::{
        error::HologramError,
        state::{
            AmmoClass, CycleTime, DamageProfile, Drone, Module, ModuleClass, Mutation,
            ProjectileSpeed,
            Rarity::{self, *},
            Shots, WeaponModuleStats,
        },
        utils::{LimitedString, RandomNumberGenerator},
    },
    anchor_lang::{require, Result},
    lazy_static::lazy_static,
};

// Totalling 100
pub const COMMON_RARITY_CHANCE: u8 = 55;
pub const UNCOMMON_RARITY_CHANCE: u8 = 20;
pub const RARE_RARITY_CHANCE: u8 = 15;
pub const FACTION_RARITY_CHANCE: u8 = 10;

pub struct LootEngine {}

impl LootEngine {
    pub fn drop_module(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
        exotic_weapon_enabled: bool,
    ) -> Result<Module> {
        let drop_rarity = Self::get_drop_rarity(rng, faction_rarity_enabled);

        // For now only offensive modules, later on make a first roll to choose if offensive, defensive, bonuses etc.
        let mut loot_table = match drop_rarity {
            Common => LT_MODULES_OFFENSIVE_COMMON.to_vec(),
            Uncommon => LT_MODULES_OFFENSIVE_UNCOMMON.to_vec(),
            Rare => LT_MODULES_OFFENSIVE_RARE.to_vec(),
            Faction => LT_MODULES_OFFENSIVE_FACTION.to_vec(),
        };

        // remove exotic drops if not enabled
        if exotic_weapon_enabled {
            loot_table = loot_table
                .into_iter()
                .filter(|m| !matches!(m.class, ModuleClass::Exotic(_)))
                .collect()
        }

        require!(
            loot_table.len() > 0 && loot_table.len() <= usize::MAX,
            HologramError::InvalidLootTable
        );
        let roll = rng.roll_dice(loot_table.len()) as usize;
        Ok(loot_table[roll - 1].clone())
    }

    pub fn drop_drone(
        rng: &mut RandomNumberGenerator,
        _faction_rarity_enabled: bool,
    ) -> Result<Drone> {
        let _roll = rng.roll_dice(100);

        Ok(Drone {
            name: LimitedString::new("Warrior II"),
            rarity: Rarity::Uncommon,
        })
    }

    pub fn drop_mutation(
        rng: &mut RandomNumberGenerator,
        _owned_mutation: &Vec<Mutation>,
    ) -> Result<Mutation> {
        let _roll = rng.roll_dice(100);

        Ok(Mutation {
            name: LimitedString::new("Fungal Growth"),
            rarity: Common,
        })
    }

    pub fn get_drop_rarity(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
    ) -> Rarity {
        let mut cumulative_chance = 0;
        let rarity_chances = [
            (COMMON_RARITY_CHANCE, Common),
            (UNCOMMON_RARITY_CHANCE, Uncommon),
            (RARE_RARITY_CHANCE, Rare),
            (
                if faction_rarity_enabled {
                    FACTION_RARITY_CHANCE
                } else {
                    0
                },
                Faction,
            ),
        ];

        let total_chance: u8 = rarity_chances.iter().map(|(chance, _)| *chance).sum();
        let roll = rng.roll_dice(total_chance as usize) as u8;

        for (chance, rarity) in rarity_chances.iter() {
            cumulative_chance += chance;
            if roll <= cumulative_chance {
                return *rarity;
            }
        }
        panic!("Invalid dice roll")
    }
}

lazy_static! {
    pub static ref LT_STARTER_WEAPON: Module = Module {
        name: LimitedString::new("Civilian Autocannon"),
        rarity: Common,
        class: ModuleClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: DamageProfile {
                em: 0,
                thermal: 0,
                kinetic: 6,
                explosive: 0,
            },
            cycle_time: CycleTime::Standard,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
    };
    pub static ref LT_MODULES_OFFENSIVE_COMMON: [Module; 3] = [
        Module {
            name: LimitedString::new("150mm Light Autocannon"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 0,
                    kinetic: 2,
                    explosive: 0,
                },
                cycle_time: CycleTime::Short,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Salvo(4),
            }),
        },
        Module {
            name: LimitedString::new("Light Ion Blaster"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 5,
                    kinetic: 5,
                    explosive: 0,
                },
                cycle_time: CycleTime::Standard,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Single,
            }),
        },
        Module {
            name: LimitedString::new("Beam Laser"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: DamageProfile {
                    em: 2,
                    thermal: 4,
                    kinetic: 0,
                    explosive: 0,
                },
                cycle_time: CycleTime::Standard,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Single,
            }),
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_UNCOMMON: [Module; 3] = [
        Module {
            name: LimitedString::new("280mm Howitzer Artillery"),
            rarity: Uncommon,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 0,
                    kinetic: 8,
                    explosive: 0,
                },
                cycle_time: CycleTime::Long,
                projectile_speed: ProjectileSpeed::SubStandard,
                shots: Shots::Salvo(2),
            }),
        },
        Module {
            name: LimitedString::new("Dual Pulse laser"),
            rarity: Uncommon,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: DamageProfile {
                    em: 1,
                    thermal: 3,
                    kinetic: 0,
                    explosive: 0,
                },
                cycle_time: CycleTime::Standard,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Salvo(2),
            }),
        },
        Module {
            name: LimitedString::new("Hydra Rockets Pod"),
            rarity: Uncommon,
            class: ModuleClass::Launcher(WeaponModuleStats {
                class: AmmoClass::Missile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 0,
                    kinetic: 1,
                    explosive: 1,
                },
                cycle_time: CycleTime::Long,
                projectile_speed: ProjectileSpeed::Slow,
                shots: Shots::Salvo(8),
            }),
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_RARE: [Module; 4] = [
        Module {
            name: LimitedString::new("125mm Railgun"),
            rarity: Rare,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 4,
                    kinetic: 10,
                    explosive: 1,
                },
                cycle_time: CycleTime::Accelerated,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Single,
            }),
        },
        Module {
            name: LimitedString::new("Focused Beam laser"),
            rarity: Rare,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: DamageProfile {
                    em: 8,
                    thermal: 12,
                    kinetic: 0,
                    explosive: 0,
                },
                cycle_time: CycleTime::Extended,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Single,
            }),
        },
        Module {
            name: LimitedString::new("Javelin LML"),
            rarity: Rare,
            class: ModuleClass::Launcher(WeaponModuleStats {
                class: AmmoClass::Missile,
                damage_profile: DamageProfile {
                    em: 0,
                    thermal: 0,
                    kinetic: 4,
                    explosive: 10,
                },
                cycle_time: CycleTime::Standard,
                projectile_speed: ProjectileSpeed::SubStandard,
                shots: Shots::Salvo(2),
            }),
        },
        Module {
            name: LimitedString::new("Entropic Desintegrator"),
            rarity: Rare,
            class: ModuleClass::Exotic(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: DamageProfile {
                    em: 2,
                    thermal: 0,
                    kinetic: 0,
                    explosive: 2,
                },
                cycle_time: CycleTime::Accelerated,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Salvo(5),
            }),
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_FACTION: [Module; 1] = [Module {
        name: LimitedString::new("Polarised Light Neutron Blaster"),
        rarity: Faction,
        class: ModuleClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: DamageProfile {
                em: 0,
                thermal: 6,
                kinetic: 6,
                explosive: 3,
            },
            cycle_time: CycleTime::Accelerated,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
    }];
}

use {
    crate::{
        error::HologramError,
        state::{
            AmmoClass, ChargeTime,
            Damage::*,
            Drone, DroneClass, DroneSize, Module, ModuleClass, Mutation, ProjectileSpeed,
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
            loot_table.retain(|m| !matches!(m.class, ModuleClass::Exotic(_)))
        }

        require!(!loot_table.is_empty(), HologramError::InvalidLootTable);
        let roll = rng.roll_dice(loot_table.len()) as usize;
        Ok(loot_table[roll - 1].clone())
    }

    pub fn drop_drone(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
    ) -> Result<Drone> {
        let drop_rarity = Self::get_drop_rarity(rng, faction_rarity_enabled);

        // For now only offensive modules, later on make a first roll to choose if offensive, defensive, bonuses etc.
        let loot_table = match drop_rarity {
            Common => LT_DRONE_OFFENSIVE_COMMON.to_vec(),
            Uncommon => LT_DRONE_OFFENSIVE_UNCOMMON.to_vec(),
            Rare => LT_DRONE_OFFENSIVE_RARE.to_vec(),
            Faction => LT_DRONE_OFFENSIVE_FACTION.to_vec(),
        };

        require!(!loot_table.is_empty(), HologramError::InvalidLootTable);
        let roll = rng.roll_dice(loot_table.len()) as usize;
        Ok(loot_table[roll - 1].clone())
    }

    pub fn drop_mutation(
        rng: &mut RandomNumberGenerator,
        _owned_mutation: &[Mutation],
    ) -> Result<Mutation> {
        let _roll = rng.roll_dice(100);

        // @TODO: Here will require a system where we can't drop the same mutation twice

        Ok(Mutation {
            name: LimitedString::new("Fungal Growth"),
            rarity: Common,
            is_active: false,
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
    // ------------------ MODULES ------------------
    pub static ref LT_STARTER_WEAPON: Module = Module {
        name: LimitedString::new("Civilian Autocannon"),
        rarity: Common,
        class: ModuleClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: [EM(0), Thermal(0), Kinetic(6), Explosive(0)],
            charge_time: ChargeTime::Standard,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
        is_active: true,
    };
    pub static ref LT_MODULES_OFFENSIVE_COMMON: [Module; 3] = [
        Module {
            name: LimitedString::new("150mm Light Autocannon"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: [EM(0), Thermal(0), Kinetic(2), Explosive(0)],
                charge_time: ChargeTime::Short,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Salvo(4),
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Light Ion Blaster"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: [EM(0), Thermal(5), Kinetic(5), Explosive(0)],
                charge_time: ChargeTime::Standard,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Single,
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Beam Laser"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: [EM(2), Thermal(4), Kinetic(0), Explosive(0)],
                charge_time: ChargeTime::Standard,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Single,
            }),
            is_active: true,
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_UNCOMMON: [Module; 3] = [
        Module {
            name: LimitedString::new("280mm Howitzer Artillery"),
            rarity: Uncommon,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: [EM(0), Thermal(0), Kinetic(8), Explosive(0)],
                charge_time: ChargeTime::Long,
                projectile_speed: ProjectileSpeed::SubStandard,
                shots: Shots::Salvo(2),
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Dual Pulse laser"),
            rarity: Uncommon,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: [EM(1), Thermal(3), Kinetic(0), Explosive(0)],
                charge_time: ChargeTime::Standard,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Salvo(2),
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Hydra Rockets Pod"),
            rarity: Uncommon,
            class: ModuleClass::Launcher(WeaponModuleStats {
                class: AmmoClass::Missile,
                damage_profile: [EM(0), Thermal(0), Kinetic(1), Explosive(1)],
                charge_time: ChargeTime::Long,
                projectile_speed: ProjectileSpeed::Slow,
                shots: Shots::Salvo(8),
            }),
            is_active: true,
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_RARE: [Module; 4] = [
        Module {
            name: LimitedString::new("125mm Railgun"),
            rarity: Rare,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Projectile,
                damage_profile: [EM(0), Thermal(4), Kinetic(10), Explosive(1)],
                charge_time: ChargeTime::Accelerated,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Single,
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Focused Beam laser"),
            rarity: Rare,
            class: ModuleClass::Turret(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: [EM(8), Thermal(12), Kinetic(0), Explosive(0)],
                charge_time: ChargeTime::Extended,
                projectile_speed: ProjectileSpeed::Fast,
                shots: Shots::Single,
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Javelin LML"),
            rarity: Rare,
            class: ModuleClass::Launcher(WeaponModuleStats {
                class: AmmoClass::Missile,
                damage_profile: [EM(0), Thermal(0), Kinetic(4), Explosive(10)],
                charge_time: ChargeTime::Standard,
                projectile_speed: ProjectileSpeed::SubStandard,
                shots: Shots::Salvo(2),
            }),
            is_active: true,
        },
        Module {
            name: LimitedString::new("Entropic Desintegrator"),
            rarity: Rare,
            class: ModuleClass::Exotic(WeaponModuleStats {
                class: AmmoClass::Energy,
                damage_profile: [EM(2), Thermal(0), Kinetic(0), Explosive(2)],
                charge_time: ChargeTime::Accelerated,
                projectile_speed: ProjectileSpeed::Standard,
                shots: Shots::Salvo(5),
            }),
            is_active: true,
        },
    ];
    pub static ref LT_MODULES_OFFENSIVE_FACTION: [Module; 1] = [Module {
        name: LimitedString::new("Polarised Light Neutron Blaster"),
        rarity: Faction,
        class: ModuleClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: [EM(0), Thermal(6), Kinetic(6), Explosive(3)],
            charge_time: ChargeTime::Accelerated,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
        is_active: true,
    }];

    // ------------------ DRONES ------------------
    pub static ref LT_DRONE_OFFENSIVE_COMMON: [Drone; 2] = [Drone {
        name: LimitedString::new("Hornet"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: [EM(0), Thermal(0), Kinetic(9), Explosive(0)],
            charge_time: ChargeTime::Standard,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new("Acolyte"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Energy,
            damage_profile: [EM(3), Thermal(0), Kinetic(0), Explosive(0)],
            charge_time: ChargeTime::Short,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_UNCOMMON: [Drone; 2] = [Drone {
        name: LimitedString::new("Vespa"),
        rarity: Uncommon,
        size: DroneSize::Medium,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: [EM(0), Thermal(0), Kinetic(18), Explosive(0)],
            charge_time: ChargeTime::Extended,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
        is_active: true,
    }
    ,Drone {
        name: LimitedString::new("Infiltrator"),
        rarity: Uncommon,
        size: DroneSize::Medium,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Energy,
            damage_profile: [EM(4), Thermal(0), Kinetic(0), Explosive(0)],
            charge_time: ChargeTime::Extended,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Salvo(4),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_RARE: [Drone; 2] = [Drone {
        name: LimitedString::new("Augmented Hornet"),
        rarity: Rare,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Projectile,
            damage_profile: [EM(0), Thermal(3), Kinetic(8), Explosive(0)],
            charge_time: ChargeTime::Accelerated,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new("Augmented Acolyte"),
        rarity: Rare,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            class: AmmoClass::Energy,
            damage_profile: [EM(6), Thermal(0), Kinetic(0), Explosive(0)],
            charge_time: ChargeTime::Short,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_FACTION: [Drone; 1] = [Drone {
        name: LimitedString::new("Prophet Mk.II"),
        rarity: Faction,
        size: DroneSize::Light,
        class: DroneClass::Launcher(WeaponModuleStats {
            class: AmmoClass::Missile,
            damage_profile: [EM(0), Thermal(1), Kinetic(1), Explosive(3)],
            charge_time: ChargeTime::Standard,
            projectile_speed: ProjectileSpeed::Standard,
            shots: Shots::Salvo(3),
        }),
        is_active: true,
    }];

    // ------------------ MUTATIONS ------------------
    pub static ref LT_MUTATIONS_UNCOMMON: [Mutation; 2] = [
        Mutation {
            name: LimitedString::new("Reverse Gravity Field"),
            rarity: Uncommon,
            is_active: false,
        },
        Mutation {
            name: LimitedString::new("Shield Polarisation"),
            rarity: Uncommon,
            is_active: false,
        }
    ];
    pub static ref LT_MUTATIONS_RARE: [Mutation; 1] = [Mutation {
        name: LimitedString::new("Nanite Outbreak"),
        rarity: Uncommon,
        is_active: false,
    }];

}

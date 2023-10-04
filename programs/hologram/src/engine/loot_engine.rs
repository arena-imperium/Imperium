use {
    crate::{
        error::HologramError,
        state::{
            Drone, DroneClass, DroneSize, Module, ModuleClass, Mutation,
            Rarity::{self, *},
            Shots, WeaponModuleStats, WeaponType,
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
    pub static ref LT_STARTER_WEAPONS: [Module; 2] = [Module {
        name: LimitedString::new("Civilian Autocannon"),
        rarity: Common,
        class: ModuleClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 11,
            shots: Shots::Single,
        }),
        is_active: true,
    },        Module {
            name: LimitedString::new("Civilian Mining Laser"),
            rarity: Common,
            class: ModuleClass::Turret(WeaponModuleStats {
                weapon_type: WeaponType::Laser,
                damage: 1,
                charge_time: 11,
                shots: Shots::Single,
            }),
            is_active: true,
        }];
    pub static ref LT_MODULES_OFFENSIVE_COMMON: [Module; 2] = [

    ];
    pub static ref LT_MODULES_OFFENSIVE_UNCOMMON: [Module; 3] = [

    ];
    pub static ref LT_MODULES_OFFENSIVE_RARE: [Module; 4] = [

    ];

    pub static ref LT_MODULES_OFFENSIVE_FACTION: [Module; 1] = [];

    // ------------------ DRONES ------------------
    pub static ref LT_DRONE_OFFENSIVE_COMMON: [Drone; 2] = [Drone {
        name: LimitedString::new("Hornet"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 10,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new("Acolyte"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 10,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_UNCOMMON: [Drone; 2] = [Drone {
        name: LimitedString::new("Vespa"),
        rarity: Uncommon,
        size: DroneSize::Medium,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 10,
            shots: Shots::Single,
        }),
        is_active: true,
    }
    ,Drone {
        name: LimitedString::new("Infiltrator"),
        rarity: Uncommon,
        size: DroneSize::Medium,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 10,
            shots: Shots::Salvo(4),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_RARE: [Drone; 2] = [Drone {
        name: LimitedString::new("Augmented Hornet"),
        rarity: Rare,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 10,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new("Augmented Acolyte"),
        rarity: Rare,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 10,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    }];

    pub static ref LT_DRONE_OFFENSIVE_FACTION: [Drone; 1] = [Drone {
        name: LimitedString::new("Prophet Mk.II"),
        rarity: Faction,
        size: DroneSize::Light,
        class: DroneClass::Turret(WeaponModuleStats {
            weapon_type: WeaponType::Missile,
            damage: 2,
            charge_time: 18,
            shots: Shots::Salvo(2),
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

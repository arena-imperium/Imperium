use {
    crate::{
        error::HologramError,
        state::{
            Bonuses, Drone, DroneClass, DroneSize, Module, ModuleClass, Mutation, Passive,
            Rarity::{self, *},
            RepairModuleStats, RepairTarget, Shots, WeaponModuleStats, WeaponType,
        },
        utils::{LimitedString, RandomNumberGenerator},
    },
    anchor_lang::{require, Result},
};

// Totalling 100
pub const COMMON_RARITY_CHANCE: u8 = 55;
pub const UNCOMMON_RARITY_CHANCE: u8 = 25;
pub const RARE_RARITY_CHANCE: u8 = 15;
pub const FACTION_RARITY_CHANCE: u8 = 5;

pub struct LootEngine {}

impl LootEngine {
    pub fn drop_module(
        rng: &mut RandomNumberGenerator,
        faction_rarity_enabled: bool,
    ) -> Result<Module> {
        let drop_rarity = Self::get_drop_rarity(rng, faction_rarity_enabled);

        // For now only offensive modules, later on make a first roll to choose if offensive, defensive, bonuses etc.
        let loot_table = match drop_rarity {
            Common => LT_MODULES_COMMON.to_vec(),
            Uncommon => LT_MODULES_OFFENSIVE_UNCOMMON.to_vec(),
            Rare => LT_MODULES_OFFENSIVE_RARE.to_vec(),
            Faction => LT_MODULES_OFFENSIVE_FACTION.to_vec(),
        };
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

// lazy_static! {
// ------------------ MODULES ---------------------------------------------------------------------
// ------------------ STARTERS --------------------------------------------------------------------
pub const LT_STARTER_OFFENSIVE_MODULES: [Module; 2] = [
    Module {
        name: LimitedString::new_const("Civilian Autocannon"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 11,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Civilian Mining Laser"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 11,
            shots: Shots::Single,
        }),
        is_active: true,
    },
];

// ------------------ COMMON ---------------------------------------------------------------------
pub const LT_MODULES_COMMON: [Module; 8] = [
    // Offensive ----------------------------------------------------------------------------------
    Module {
        name: LimitedString::new_const("Pulse Laser"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 10,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Dual Pulse Laser"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 21,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Slicer"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Beam,
            damage: 2,
            charge_time: 18,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("175mm Artillery"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 3,
            charge_time: 28,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Light Missile Launcher I"),
        rarity: Common,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Missile,
            damage: 2,
            charge_time: 19,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    // Other ----------------------------------------------------------------------------------
    Module {
        name: LimitedString::new_const("Compact Shield Booster"),
        rarity: Rare,
        class: ModuleClass::Repairer(
            Bonuses {
                hull_hitpoints: 0,
                shield_layers: 1,
                dodge_chance: 0,
            },
            RepairModuleStats {
                repair_amount: 1,
                charge_time: 16,
                target: RepairTarget::Shield,
            },
        ),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Capacitative Armor"),
        rarity: Rare,
        class: ModuleClass::Capacitative(
            Bonuses {
                hull_hitpoints: 5,
                shield_layers: 0,
                dodge_chance: 0,
            },
            Passive::CapacitativeRepair {
                threshold: 5,
                repair_amount: 3,
                target: RepairTarget::Hull,
            },
        ),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Capacitative Shield Battery"),
        rarity: Uncommon,
        class: ModuleClass::Capacitative(
            Bonuses {
                hull_hitpoints: 0,
                shield_layers: 1,
                dodge_chance: 0,
            },
            Passive::CapacitativeRepair {
                threshold: 5,
                repair_amount: 3,
                target: RepairTarget::Shield,
            },
        ),
        is_active: true,
    },
];

// ------------------ UNCOMMON ---------------------------------------------------------------------
pub const LT_MODULES_OFFENSIVE_UNCOMMON: [Module; 3] = [
    Module {
        name: LimitedString::new_const("Heavy Pulse Laser"),
        rarity: Uncommon,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 2,
            charge_time: 16,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("125mm Dual Autocannon"),
        rarity: Uncommon,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 18,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Assault Missile Launcher"),
        rarity: Uncommon,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Missile,
            damage: 4,
            charge_time: 30,
            shots: Shots::Single,
        }),
        is_active: true,
    },
];

pub const LT_MODULES_OFFENSIVE_RARE: [Module; 3] = [
    Module {
        name: LimitedString::new_const("280mm 'Howitzer' Artillery"),
        rarity: Rare,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 5,
            charge_time: 30,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("'Halberd' Slicer"),
        rarity: Rare,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 4,
            charge_time: 23,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Module {
        name: LimitedString::new_const("Rapid Light Missile Launcher"),
        rarity: Rare,
        class: ModuleClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Missile,
            damage: 2,
            charge_time: 12,
            shots: Shots::Single,
        }),
        is_active: true,
    },
];

pub const LT_MODULES_OFFENSIVE_FACTION: [Module; 1] = [Module {
    name: LimitedString::new_const("Polarised Repeating Laser"),
    rarity: Faction,
    class: ModuleClass::Weapon(WeaponModuleStats {
        weapon_type: WeaponType::Laser,
        damage: 1,
        charge_time: 16,
        shots: Shots::Salvo(3),
    }),
    is_active: true,
}];

// ------------------ DRONES ------------------
pub const LT_DRONE_OFFENSIVE_COMMON: [Drone; 2] = [
    Drone {
        name: LimitedString::new_const("Hornet"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 10,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new_const("Acolyte"),
        rarity: Common,
        size: DroneSize::Light,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 20,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    },
];

pub const LT_DRONE_OFFENSIVE_UNCOMMON: [Drone; 2] = [
    Drone {
        name: LimitedString::new_const("Augmented Hornet"),
        rarity: Uncommon,
        size: DroneSize::Light,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 1,
            charge_time: 9,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new_const("Augmented Acolyte"),
        rarity: Uncommon,
        size: DroneSize::Light,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 17,
            shots: Shots::Salvo(2),
        }),
        is_active: true,
    },
];

pub const LT_DRONE_OFFENSIVE_RARE: [Drone; 2] = [
    Drone {
        name: LimitedString::new_const("Vespa"),
        rarity: Rare,
        size: DroneSize::Medium,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Projectile,
            damage: 2,
            charge_time: 13,
            shots: Shots::Single,
        }),
        is_active: true,
    },
    Drone {
        name: LimitedString::new_const("Infiltrator"),
        rarity: Rare,
        size: DroneSize::Medium,
        class: DroneClass::Weapon(WeaponModuleStats {
            weapon_type: WeaponType::Laser,
            damage: 1,
            charge_time: 18,
            shots: Shots::Salvo(3),
        }),
        is_active: true,
    },
];

pub const LT_DRONE_OFFENSIVE_FACTION: [Drone; 1] = [Drone {
    name: LimitedString::new_const("Prophet Mk.II"),
    rarity: Faction,
    size: DroneSize::Light,
    class: DroneClass::Weapon(WeaponModuleStats {
        weapon_type: WeaponType::Missile,
        damage: 2,
        charge_time: 30,
        shots: Shots::Salvo(3),
    }),
    is_active: true,
}];

// ------------------ MUTATIONS ------------------
pub const LT_MUTATIONS_UNCOMMON: [Mutation; 1] = [Mutation {
    name: LimitedString::new_const("Nanite Coating"),
    rarity: Uncommon,
    is_active: false,
}];
pub const LT_MUTATIONS_RARE: [Mutation; 1] = [Mutation {
    name: LimitedString::new_const("Nanite Outbreak"),
    rarity: Uncommon,
    is_active: false,
}];

// }

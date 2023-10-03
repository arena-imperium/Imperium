use super::PowerUp;

use {
    super::PowerupKind,
    crate::{
        state::{AmmoClass, Damage, ModuleClass},
        utils::LimitedString,
    },
};

// Derived from a power up to be used in the fight engine
pub struct PassivePowerup {
    pub name: LimitedString,
    pub modifiers: Vec<PassiveModifier>,
    pub og_kind: PowerupKind,
}

impl PassivePowerup {
    pub fn new(powerup: Box<dyn PowerUp>) -> Self {
        Self {
            name: powerup.get_name(),
            modifiers: powerup.get_modifiers(),
            og_kind: powerup.get_kind(),
        }
    }
}

// Modifiers from passive powerups
pub enum PassiveModifier {
    ReduceModuleChargeTime { class: ModuleClass, amount: u8 },
    IncreaseProjectileSpeed { class: AmmoClass, amount: u8 },
    // % chance for the damage from target to instead repair the Shield
    DamageRechargeShield { damage_type: Damage, chance: u8 },
}

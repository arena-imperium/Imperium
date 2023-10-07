use {
    super::{PowerUp, PowerupKind},
    crate::{state::WeaponType, utils::LimitedString},
};

// Derived from a power up to be used in the fight engine
#[derive(Debug)]
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

#[derive(Debug)]
pub enum PassiveModifier {
    // % chance to cancel a type of damage
    DamageAbsorbtion { weapon_type: WeaponType, chance: u8 },
}

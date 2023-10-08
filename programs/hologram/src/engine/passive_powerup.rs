use {
    super::{Effect, PowerUp, PowerupKind},
    crate::{state::Bonuses, utils::LimitedString},
};

// Derived from a power up to be used in the fight engine
#[derive(Debug)]
pub struct PassivePowerup {
    pub name: LimitedString,
    pub effects: Vec<Effect>,
    pub bonuses: Option<Bonuses>,
    pub og_kind: PowerupKind,
}

impl PassivePowerup {
    pub fn new(powerup: Box<dyn PowerUp>) -> Self {
        Self {
            name: powerup.get_name(),
            effects: powerup.get_effects(),
            bonuses: powerup.get_bonuses(),
            og_kind: powerup.get_kind(),
        }
    }
}

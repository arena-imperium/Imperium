use {
    super::{PowerUp, PowerupKind},
    crate::{
        state::{Damage, Shots},
        utils::LimitedString,
    },
};

#[derive(Debug, Clone)]
// Derived from a power up to be used in the fight engine
pub struct ActivePowerup {
    pub name: LimitedString,
    pub accumulated_charge: u8,
    pub charge_time: u8,
    pub effects: Vec<ActiveEffect>,
    pub og_kind: PowerupKind,
}

impl ActivePowerup {
    pub fn new(powerup: Box<dyn PowerUp>) -> Self {
        Self {
            name: powerup.get_name(),
            accumulated_charge: 0,
            charge_time: powerup.get_charge_time().unwrap(),
            effects: powerup.get_effects(),
            og_kind: powerup.get_kind(),
        }
    }

    // charge the module and return true if it's been activated
    pub fn charge_and_activate(&mut self, amount: u8) -> bool {
        self.accumulated_charge += amount;
        self.activate()
    }

    pub fn activate(&mut self) -> bool {
        if self.is_charged() {
            self.accumulated_charge -= self.charge_time;
            return true;
        }
        return false;
    }

    fn is_charged(&self) -> bool {
        self.accumulated_charge >= self.charge_time
    }
}

#[derive(Debug, Clone)]
// Effect of active powerups
pub enum ActiveEffect {
    Fire {
        damages: [Damage; 4],
        shots: Shots,
        projectile_speed: u8,
    },
    RepairHull {
        amount: u8,
    },
    RepairArmor {
        amount: u8,
    },
    RepairShield {
        amount: u8,
    },
    Jam {
        charge_burn: u8,
    },
    // Composite effect represent a chance to apply one of two effects
    Composite {
        effect1: Box<ActiveEffect>,
        effect2: Box<ActiveEffect>,
        probability1: u8,
        probability2: u8,
    },
}

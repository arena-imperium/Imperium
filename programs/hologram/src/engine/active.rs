use {
    super::{PowerUp, PowerupKind},
    crate::{
        state::{RepairTarget, Shots, WeaponType},
        utils::LimitedString,
    },
};

#[derive(Debug, Clone)]
// Derived from a power up to be used in the fight engine
pub struct ActivePowerup {
    pub name: LimitedString,
    // how much charge the module has accumulated
    pub accumulated_charge: u8,
    // how long it take to activate
    pub charge_time: u8,
    // what the powerup does
    pub effects: Vec<ActiveEffect>,
    // the base type of the power up for filtering/ui purposes
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
    // shooting at the opponent. Support all weapon type
    Fire {
        damage: u8,
        shots: Shots,
        weapon_type: WeaponType,
    },
    Repair {
        target: RepairTarget,
        amount: u8,
    },
    // attempt to disrupt opponent active powerups
    Jam {
        chance: u8,
        charge_burn: u8,
    },
    // Composite effect representing a chance to apply one of two effects
    Composite {
        effect1: Box<ActiveEffect>,
        effect2: Box<ActiveEffect>,
        probability1: u8,
        probability2: u8,
    },
}

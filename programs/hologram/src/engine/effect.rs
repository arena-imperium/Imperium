use {
    super::SpaceShipBattleCard,
    crate::state::{RepairTarget, Shots, WeaponType},
    core::fmt,
    std::sync::Arc,
};

#[derive(Debug, Clone)]
pub enum Effect {
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
    // [Higher order Effects]
    // HO effect representing a chance to apply an effect or do nothing
    Chance {
        probability: u8,
        effect: Box<Effect>,
    },
    // HO effect representing a chance to apply one of two effects
    Composite {
        effect1: Box<Effect>,
        effect2: Box<Effect>,
        probability1: u8,
        probability2: u8,
    },
    // HO effect that only happen when the condition function return true
    Conditionnal {
        condition: ConditionFn,
        effect: Box<Effect>,
    },
}

pub struct ConditionFn {
    pub func: Arc<dyn Fn(&SpaceShipBattleCard) -> bool + Send + Sync>,
}

impl Clone for ConditionFn {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

impl fmt::Debug for ConditionFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Condition function")
    }
}

#[cfg(any(test, feature = "testing"))]
impl fmt::Display for Shots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shots::Single => write!(f, "single shot"),
            Shots::Salvo(n) => write!(f, "{} shots", n),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl fmt::Display for RepairTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepairTarget::Hull => write!(f, "Hull HP"),
            RepairTarget::Shield => write!(f, "Shield layer"),
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl fmt::Display for WeaponType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeaponType::Laser => write!(f, "Laser"),
            WeaponType::Missile => write!(f, "Missile"),
            WeaponType::Projectile => write!(f, "Projectile"),
            WeaponType::Plasma => write!(f, "Plasma"),
        }
    }
}

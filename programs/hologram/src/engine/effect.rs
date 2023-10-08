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
    // Composite effect representing a chance to apply one of two effects
    Composite {
        effect1: Box<Effect>,
        effect2: Box<Effect>,
        probability1: u8,
        probability2: u8,
    },
    // Conditionnal effect that only happen when the condition function return true
    Conditionnal {
        condition: ConditionFn,
        effect: Box<Effect>,
    },
    //
    // % chance to cancel a type of damage
    DamageAbsorbtion {
        weapon_type: WeaponType,
        chance: u8,
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

#[cfg(any(test, feature = "testing"))]
impl fmt::Debug for ConditionFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Condition function")
    }
}

#[cfg(any(test, feature = "testing"))]
impl fmt::Display for Shots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shots::Single => write!(f, "Single shot"),
            Shots::Salvo(n) => write!(f, "{} shots salvo", n),
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

#[cfg(any(test, feature = "testing"))]
impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Fire {
                damage,
                shots,
                weapon_type,
            } => write!(
                f,
                "fires a {} of {} damage ({})",
                shots, damage, weapon_type
            ),
            Effect::Repair { target, amount } => {
                write!(f, "repairs {} {}", amount, target)
            }
            Effect::Jam {
                chance,
                charge_burn,
            } => write!(
                f,
                "jam with {}% chance and {} charge burn",
                chance, charge_burn
            ),
            Effect::Composite {
                effect1,
                effect2,
                probability1,
                probability2,
            } => write!(
                f,
                "Composite effect with {}% chance of {} and {}% chance of {}",
                probability1, effect1, probability2, effect2
            ),
            Effect::Conditionnal { condition, effect } => {
                write!(
                    f,
                    "Conditionnal effect with condition [redacted] and effect {}",
                    effect
                )
            }
            Effect::DamageAbsorbtion {
                weapon_type,
                chance,
            } => write!(
                f,
                "Damage absorbtion of {} with {}% chance",
                weapon_type, chance
            ),
        }
    }
}

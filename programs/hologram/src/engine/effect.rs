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

impl fmt::Debug for ConditionFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Condition function")
    }
}

impl Clone for ConditionFn {
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}
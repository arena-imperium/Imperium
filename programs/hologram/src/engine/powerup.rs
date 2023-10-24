use {
    super::Effect,
    crate::{
        engine::ConditionFn,
        state::{Bonuses, Drone, DroneClass, DroneSize, Module, ModuleClass, Mutation, Passive},
        utils::LimitedString,
    },
    std::sync::Arc,
};

// tag trait for Modules, Drones and Mutations
pub trait PowerUp {
    fn get_name(&self) -> LimitedString;
    // for active power-ups, how many charge it takes before being activated
    fn is_active(&self) -> bool;
    // activation delay mechanism for active modules
    fn get_charge_time(&self) -> Option<u8>;
    // reactivation delay mechanism for passive modules
    fn get_heat(&self) -> Option<u8>;
    // get what happen on activation
    fn get_effect(&self) -> Effect;
    // get bonuses
    fn get_bonuses(&self) -> Option<Bonuses>;
    fn get_kind(&self) -> PowerupKind;
}

impl PowerUp for Module {
    fn get_name(&self) -> LimitedString {
        self.name.clone()
    }

    fn is_active(&self) -> bool {
        match &self.class {
            ModuleClass::Weapon(_) | ModuleClass::Repairer(_, _) => true,
            ModuleClass::Capacitative(_, _) => false,
        }
    }

    fn get_charge_time(&self) -> Option<u8> {
        match &self.class {
            ModuleClass::Weapon(wms) => Some(wms.charge_time as u8),
            ModuleClass::Repairer(_, rms) => Some(rms.charge_time as u8),
            ModuleClass::Capacitative(_, _) => None,
        }
    }

    fn get_heat(&self) -> Option<u8> {
        match &self.class {
            ModuleClass::Weapon(_) => None,
            ModuleClass::Repairer(_, _) => None,
            ModuleClass::Capacitative(_, passive) => {
                let Passive::CapacitativeRepair {
                    recent_damage_threshold: _,
                    heat,
                    repair_amount: _,
                    target: _,
                } = &passive;
                Some(*heat)
            }
        }
    }

    fn get_effect(&self) -> Effect {
        match &self.class {
            ModuleClass::Weapon(wms) => Effect::Fire {
                damage: wms.damage,
                shots: wms.shots,
                weapon_type: wms.weapon_type,
            },
            ModuleClass::Repairer(_, rms) => Effect::Repair {
                target: rms.target,
                amount: rms.repair_amount,
            },
            ModuleClass::Capacitative(_, passive) => {
                let Passive::CapacitativeRepair {
                    recent_damage_threshold: threshold,
                    heat: _,
                    repair_amount,
                    target,
                } = &passive;
                let threshold_clone = threshold.clone();
                Effect::Conditionnal {
                    condition: ConditionFn {
                        func: Arc::new(move |sbc| sbc.recent_hull_damage() >= threshold_clone),
                    },
                    effect: Box::new(Effect::Repair {
                        target: *target,
                        amount: *repair_amount,
                    }),
                }
            }
        }
    }

    fn get_bonuses(&self) -> Option<Bonuses> {
        match &self.class {
            ModuleClass::Weapon(_) => None,
            ModuleClass::Repairer(bonuses, _) | ModuleClass::Capacitative(bonuses, _) => {
                Some(bonuses.clone())
            }
        }
    }

    fn get_kind(&self) -> PowerupKind {
        PowerupKind::Module {
            class: self.class.clone(),
        }
    }
}

impl PowerUp for Drone {
    fn get_name(&self) -> LimitedString {
        self.name.clone()
    }

    fn is_active(&self) -> bool {
        match &self.class {
            DroneClass::Weapon(_) | DroneClass::ECM(_) => true,
        }
    }

    fn get_charge_time(&self) -> Option<u8> {
        match &self.class {
            DroneClass::Weapon(wms) => Some(wms.charge_time as u8),
            DroneClass::ECM(jms) => Some(jms.charge_time as u8),
        }
    }

    fn get_heat(&self) -> Option<u8> {
        None
    }

    fn get_effect(&self) -> Effect {
        match &self.class {
            DroneClass::Weapon(wms) => Effect::Fire {
                damage: wms.damage,
                shots: wms.shots,
                weapon_type: wms.weapon_type,
            },
            DroneClass::ECM(jms) => Effect::Jam {
                chance: jms.chance,
                charge_burn: jms.charge_burn,
            },
        }
    }

    fn get_bonuses(&self) -> Option<Bonuses> {
        None
    }

    fn get_kind(&self) -> PowerupKind {
        PowerupKind::Drone {
            class: self.class.clone(),
            size: self.size.clone(),
        }
    }
}

impl PowerUp for Mutation {
    fn get_name(&self) -> LimitedString {
        self.name.clone()
    }
    fn is_active(&self) -> bool {
        panic!("Not implemented")
    }
    fn get_charge_time(&self) -> Option<u8> {
        panic!("Not implemented")
    }
    fn get_heat(&self) -> Option<u8> {
        panic!("Not implemented")
    }
    fn get_effect(&self) -> Effect {
        panic!("Not implemented")
    }
    fn get_bonuses(&self) -> Option<Bonuses> {
        panic!("Not implemented")
    }
    fn get_kind(&self) -> PowerupKind {
        PowerupKind::Mutation
    }
}

#[derive(Debug, Clone)]
pub enum PowerupKind {
    Module { class: ModuleClass },
    Drone { class: DroneClass, size: DroneSize },
    Mutation,
}

use {
    super::{ActiveEffect, PassiveModifier},
    crate::{
        state::{Drone, DroneClass, DroneSize, Module, ModuleClass, Mutation},
        utils::LimitedString,
    },
};

// tag trait for Modules, Drones and Mutations
pub trait PowerUp {
    fn get_name(&self) -> LimitedString;
    fn is_active(&self) -> bool;
    fn get_charge_time(&self) -> Option<u8>;
    fn get_effects(&self) -> Vec<ActiveEffect>;
    fn get_modifiers(&self) -> Vec<PassiveModifier>;
    fn get_kind(&self) -> PowerupKind;
}

impl PowerUp for Module {
    fn get_name(&self) -> LimitedString {
        self.name.clone()
    }

    fn is_active(&self) -> bool {
        match &self.class {
            ModuleClass::Weapon(_) | ModuleClass::Repairer(_) => true,
            ModuleClass::ShieldAmplifier | ModuleClass::TrackingComputer => false,
        }
    }

    fn get_charge_time(&self) -> Option<u8> {
        match &self.class {
            ModuleClass::Weapon(m) => Some(m.charge_time as u8),
            ModuleClass::Repairer(m) => Some(m.charge_time as u8),
            ModuleClass::ShieldAmplifier | ModuleClass::TrackingComputer => None,
        }
    }
    fn get_effects(&self) -> Vec<ActiveEffect> {
        match &self.class {
            ModuleClass::Weapon(wms) => {
                vec![ActiveEffect::Fire {
                    damage: wms.damage,
                    shots: wms.shots,
                    weapon_type: wms.weapon_type,
                }]
            }
            ModuleClass::Repairer(rms) => vec![ActiveEffect::Repair {
                target: rms.target,
                amount: rms.repair_amount,
            }],
            ModuleClass::ShieldAmplifier | ModuleClass::TrackingComputer => vec![],
        }
    }
    fn get_modifiers(&self) -> Vec<PassiveModifier> {
        match &self.class {
            ModuleClass::Weapon(_) | ModuleClass::Repairer(_) => vec![],
            ModuleClass::ShieldAmplifier => todo!(),
            ModuleClass::TrackingComputer => todo!(),
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
    fn get_effects(&self) -> Vec<ActiveEffect> {
        match &self.class {
            DroneClass::Weapon(wms) => {
                vec![ActiveEffect::Fire {
                    damage: wms.damage,
                    shots: wms.shots,
                    weapon_type: wms.weapon_type,
                }]
            }
            DroneClass::ECM(jms) => vec![ActiveEffect::Jam {
                chance: jms.chance,
                charge_burn: jms.charge_burn,
            }],
        }
    }
    fn get_modifiers(&self) -> Vec<PassiveModifier> {
        todo!()
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
    fn get_effects(&self) -> Vec<ActiveEffect> {
        panic!("Not implemented")
    }
    fn get_modifiers(&self) -> Vec<PassiveModifier> {
        todo!()
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

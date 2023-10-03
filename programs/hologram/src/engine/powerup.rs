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
            ModuleClass::Turret(_)
            | ModuleClass::Launcher(_)
            | ModuleClass::Exotic(_)
            | ModuleClass::ShieldBooster(_)
            | ModuleClass::ArmorRepairer(_)
            | ModuleClass::HullRepairer(_) => true,
            ModuleClass::ShieldAmplifier
            | ModuleClass::NaniteCoating
            | ModuleClass::TrackingComputer
            | ModuleClass::Gyrostabilizer
            | ModuleClass::TrackingDisruptor => false,
        }
    }

    fn get_charge_time(&self) -> Option<u8> {
        match &self.class {
            ModuleClass::Turret(m) | ModuleClass::Launcher(m) | ModuleClass::Exotic(m) => {
                Some(m.charge_time as u8)
            }
            ModuleClass::ShieldBooster(m)
            | ModuleClass::ArmorRepairer(m)
            | ModuleClass::HullRepairer(m) => Some(m.charge_time as u8),
            ModuleClass::ShieldAmplifier
            | ModuleClass::NaniteCoating
            | ModuleClass::TrackingComputer
            | ModuleClass::Gyrostabilizer
            | ModuleClass::TrackingDisruptor => None,
        }
    }
    fn get_effects(&self) -> Vec<ActiveEffect> {
        match &self.class {
            ModuleClass::Turret(wms) | ModuleClass::Launcher(wms) | ModuleClass::Exotic(wms) => {
                vec![ActiveEffect::Fire {
                    damages: wms.damage_profile,
                    shots: wms.shots,
                    projectile_speed: wms.projectile_speed as u8,
                }]
            }
            ModuleClass::ShieldBooster(rms) => vec![ActiveEffect::RepairShield {
                amount: rms.repair_amount,
            }],
            ModuleClass::ArmorRepairer(rms) => vec![ActiveEffect::RepairArmor {
                amount: rms.repair_amount,
            }],
            ModuleClass::HullRepairer(rms) => vec![ActiveEffect::RepairHull {
                amount: rms.repair_amount,
            }],
            ModuleClass::ShieldAmplifier
            | ModuleClass::NaniteCoating
            | ModuleClass::TrackingComputer
            | ModuleClass::Gyrostabilizer
            | ModuleClass::TrackingDisruptor => vec![],
        }
    }
    fn get_modifiers(&self) -> Vec<PassiveModifier> {
        match &self.class {
            ModuleClass::Turret(_)
            | ModuleClass::Launcher(_)
            | ModuleClass::Exotic(_)
            | ModuleClass::ShieldBooster(_)
            | ModuleClass::ArmorRepairer(_)
            | ModuleClass::HullRepairer(_) => vec![],
            ModuleClass::ShieldAmplifier => todo!(),
            ModuleClass::NaniteCoating => todo!(),
            ModuleClass::TrackingComputer => todo!(),
            ModuleClass::Gyrostabilizer => todo!(),
            ModuleClass::TrackingDisruptor => todo!(),
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
            DroneClass::Turret(_)
            | DroneClass::Launcher(_)
            | DroneClass::Exotic(_)
            | DroneClass::ECM(_) => true,
        }
    }

    fn get_charge_time(&self) -> Option<u8> {
        match &self.class {
            DroneClass::Turret(wms) | DroneClass::Launcher(wms) | DroneClass::Exotic(wms) => {
                Some(wms.charge_time as u8)
            }
            DroneClass::ECM(jms) => Some(jms.charge_time as u8),
        }
    }
    fn get_effects(&self) -> Vec<ActiveEffect> {
        match &self.class {
            DroneClass::Turret(wms) | DroneClass::Launcher(wms) | DroneClass::Exotic(wms) => {
                vec![ActiveEffect::Fire {
                    damages: wms.damage_profile,
                    shots: wms.shots,
                    projectile_speed: wms.projectile_speed as u8,
                }]
            }
            DroneClass::ECM(jms) => vec![ActiveEffect::Jam {
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

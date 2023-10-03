use crate::{utils::RandomNumberGenerator, BASE_CRIT_CHANCE, BASE_FUMBLE_CHANCE};

use {
    super::{ActiveEffect, ActivePowerup, PassiveModifier, PassivePowerup, PowerupKind},
    crate::state::{Damage, HitPoints, ModuleClass, Shots},
};

pub struct SpaceShipBattleCard {
    // stats ----------------------------------------
    // Hitpoints
    pub hull_hitpoints: HitPoints,
    pub armor_hitpoints: HitPoints,
    pub shield_hitpoints: HitPoints,
    // similar to MTG. Determine who will hit first
    pub celerity: u8,
    // reductions in charge times
    pub turret_charge_time_reduction: u8,
    // bonus to projectile speed
    pub turret_bonus_projectile_speed: u8,
    // chance to avoid attacks/jams
    pub dodge_chance: u8,
    pub jamming_nullifying_chance: u8,
    // powerups -------------------------------------
    // they are kept in order to cycle through them during the actual fight
    pub active_powerups: Vec<ActivePowerup>,
    // their effects are already compounded in the stas above. they are kept for reference
    pub passive_powerups: Vec<PassivePowerup>,
}

impl SpaceShipBattleCard {
    // A spaceship is defeated when his Hull HP reaches 0
    pub fn is_defeated(&self) -> bool {
        self.hull_hitpoints.current == 0
    }

    pub fn fire_at(
        &mut self,
        target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
        damage: [Damage; 4],
        shots: Shots,
        projectile_speed: u8,
    ) {
        // hit roll
        let hit_roll = rng.roll_dice(100);
        let did_hit = hit_roll >= target.dodge_chance as u64;

        match shots {
            Shots::Single => {
                if !did_hit {
                    return;
                }
                let crit_roll = rng.roll_dice(100);
                let speed_modifier = projectile_speed / 100; // Convert projectile speed to a percentage
                let crit_chance = BASE_CRIT_CHANCE + speed_modifier; // Add speed modifier to base crit chance
                let did_crit = crit_roll >= (100 - crit_chance as u64);                
                target.apply_damage(damage, did_crit);
            }
            Shots::Salvo(shots) => {
                let shots = match did_hit {
                    true => shots,
                    false => shots / 2,
                };
                for dmg in damage.iter_mut() {
                    dmg *= shots;
                }
            },
        };
    }

    pub fn apply_damage(&mut self, damage: [Damage; 4], did_crit: bool) {
        if self.shield_hitpoints.current > 0 {
            self.shield_hitpoints.current -= damage[0];
            if self.shield_hitpoints.current < 0 {
                self.shield_hitpoints.current = 0;
            }
        } else if self.armor_hitpoints.current > 0 {
            self.armor_hitpoints.current -= damage[1];
            if self.armor_hitpoints.current < 0 {
                self.armor_hitpoints.current = 0;
            }
        } else {
            self.hull_hitpoints.current -= damage[2];
            if self.hull_hitpoints.current < 0 {
                self.hull_hitpoints.current = 0;
            }
        }
    }

    // /// go through each passive powerup
    // /// for each of them go through each of their modifiers
    // /// and apply these modifiers to the active powerups effects
    // pub fn apply_passive_powerups(&mut self) {
    //     self.passive_powerups.iter().for_each(|p| {
    //         p.modifiers.iter().for_each(|e| match e {
    //             PassiveModifier::ReduceModuleChargeTime { class, amount } => {
    //                 // for active modules of the given ModuleClass, reduce charge_time by amount
    //                 self.active_powerups.iter_mut().for_each(|a| {
    //                     if let PowerupKind::Module { class: mc } = &a.og_kind {
    //                         if mc == class {
    //                             a.charge_time -= amount;
    //                         }
    //                     }
    //                 });
    //             }
    //             PassiveModifier::IncreaseProjectileSpeed { class, amount } => {
    //                 // for active modules of the given AmmoClass, increase projectile_speed by amount
    //                 self.active_powerups.iter_mut().for_each(|a| {
    //                     if let PowerupKind::Module { class: mc } = &a.og_kind {
    //                         match mc {
    //                             ModuleClass::Turret(wms)
    //                             | ModuleClass::Launcher(wms)
    //                             | ModuleClass::Exotic(wms) => {
    //                                 if wms.class == *class {
    //                                     a.effects.iter_mut().for_each(|e| match e {
    //                                         ActiveEffect::Fire {
    //                                             damages: _,
    //                                             shots: _,
    //                                             projectile_speed,
    //                                         } => {
    //                                             *projectile_speed += *amount;
    //                                         }
    //                                         _ => {}
    //                                     });
    //                                 }
    //                             }
    //                             _ => {}
    //                         };
    //                     }
    //                 });
    //             }
    //             PassiveModifier::ModuleDamageRechargeShield {
    //                 damage_type,
    //                 chance,
    //             } => {
    //                 self.active_powerups.iter_mut().for_each(|a| {
    //                     if let PowerupKind::Module { class: mc } = &a.og_kind {
    //                         match mc {
    //                             ModuleClass::Turret(wms)
    //                             | ModuleClass::Launcher(wms)
    //                             | ModuleClass::Exotic(wms) => {
    //                                 if wms.damage_profile.contains(damage_type) {
    //                                     a.effects.iter_mut().for_each(|e| match e {
    //                                         ActiveEffect::Fire {
    //                                             damages: damages,
    //                                             shots: shots,
    //                                             projectile_speed: _,
    //                                         } => {

    //                                             *e = ActiveEffect::Composite {
    //                                                 effect1: Box::new(ActiveEffect::Fire {
    //                                                     damages: wms.damage_profile,
    //                                                     shots: wms.shots,
    //                                                     projectile_speed: wms.projectile_speed
    //                                                         as u8,
    //                                                 }),
    //                                                 effect2: Box::new(ActiveEffect::RepairShield {
    //                                                     amount: ,
    //                                                 }),
    //                                                 probability1: 100 - chance,
    //                                                 probability2: *chance,
    //                                             };
    //                                         }
    //                                         _ => {}
    //                                     });
    //                                 }
    //                             }
    //                             _ => {}
    //                         };
    //                     }
    //                 });
    //             }
    //         });
    //     });
    // }
}

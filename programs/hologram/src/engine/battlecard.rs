use {
    super::{ActivePowerup, PassivePowerup},
    crate::{
        state::{HitPoints, Shots, WeaponType},
        utils::RandomNumberGenerator,
    },
};

pub struct SpaceShipBattleCard {
    // stats ----------------------------------------
    // Hitpoints
    pub hull_hitpoints: HitPoints,
    pub shield_layers: HitPoints,
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
        self.hull_hitpoints.depleted()
    }

    pub fn fire_at(
        &mut self,
        target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
        damage: u8,
        shots: Shots,
        weapon_type: WeaponType,
    ) {
        // hit roll (beams cannot be dodged)
        if weapon_type != WeaponType::Beam {
            let hit_roll = rng.roll_dice(100);
            let did_hit = hit_roll >= target.dodge_chance as u64;
            if !did_hit {
                return;
            }
        }

        match shots {
            Shots::Single | Shots::Salvo(1) => target.apply_damage(damage, weapon_type),
            Shots::Salvo(shots) => {
                for _ in 0..shots {
                    target.apply_damage(damage, weapon_type);
                }
            }
        };
    }

    pub fn jam(
        &mut self,
        target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
        chance: u8,
        charge_burn: u8,
    ) {
        let jam_chance = chance.saturating_sub(target.jamming_nullifying_chance);
        if rng.roll_dice(100 as usize) <= jam_chance as u64 {
            // filter powerups with charge only
            let active_powerups_with_charge_indexes: Vec<usize> = target
                .active_powerups
                .iter_mut()
                .enumerate()
                .filter(|(_, a)| a.accumulated_charge != 0)
                .map(|(i, _)| i)
                .collect();

            if active_powerups_with_charge_indexes.is_empty() {
                return;
            }

            let roll = rng.roll_dice(active_powerups_with_charge_indexes.len());
            let index = active_powerups_with_charge_indexes[roll as usize];
            target.active_powerups[index].accumulated_charge = target.active_powerups[index]
                .accumulated_charge
                .saturating_sub(charge_burn);
        }
    }

    pub fn apply_damage(&mut self, damage: u8, weapon_type: WeaponType) {
        match weapon_type {
            WeaponType::Projectile => {
                self.hull_hitpoints.deplete(damage);
            }
            WeaponType::Missile => {
                self.hull_hitpoints.deplete(damage);
            }
            WeaponType::Laser => {
                if self.shield_layers.depleted() {
                    self.hull_hitpoints.deplete(damage);
                } else {
                    self.shield_layers.deplete(1);
                }
            }
            WeaponType::Beam => {
                // only inflicts damage if shields are down
                if self.shield_layers.depleted() {
                    self.hull_hitpoints.deplete(damage);
                }
            }
        }
    }
}

use {
    super::{ActivePowerup, PassivePowerup, PowerUp},
    crate::{
        state::{HitPoints, Shots, SpaceShip, WeaponType},
        utils::RandomNumberGenerator,
        BASE_DODGE_CHANCE, BASE_HULL_HITPOINTS, BASE_JAMMING_NULLIFYING_CHANCE, BASE_SHIELD_LAYERS,
        DODGE_CHANCE_CAP, JAMMING_NULLIFYING_CHANCE_CAP,
    },
    std::cmp::max,
};

// Note: Recentely == 5 last turns
#[derive(Debug)]
pub struct SpaceShipBattleCard {
    pub name: String,
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
    // data -----------------------------------------
    // Note: data internal to the game engine that is updated along the match
    // for CapacitativeArmor
    //
    // Stores last 5 turns damages to the hull
    pub recent_hull_damage_per_turn: Vec<u8>,
}

impl SpaceShipBattleCard {
    // Initialize a battlecard from a spaceship
    pub fn new(spaceship: &SpaceShip) -> Self {
        // convert all modules, drones, mutations to PowerUp
        let powerups: Vec<Box<dyn PowerUp>> = spaceship
            .modules
            .iter()
            .map(|item| Box::new(item.clone()) as Box<dyn PowerUp>)
            .chain(
                spaceship
                    .drones
                    .iter()
                    .map(|item| Box::new(item.clone()) as Box<dyn PowerUp>),
            )
            .chain(
                spaceship
                    .mutations
                    .iter()
                    .map(|item| Box::new(item.clone()) as Box<dyn PowerUp>),
            )
            .collect();

        // initialize stats
        let mut hull_hitpoints = HitPoints::init(BASE_HULL_HITPOINTS);
        let mut shield_layers = HitPoints::init(BASE_SHIELD_LAYERS);
        let mut dodge_chance = BASE_DODGE_CHANCE;
        let mut jamming_nullifying_chance = BASE_JAMMING_NULLIFYING_CHANCE;
        // apply all bonuses from powerups
        powerups
            .iter()
            .filter_map(|p| p.get_bonuses())
            .for_each(|bonuses| {
                hull_hitpoints.increase_max(bonuses.hull_hitpoints);
                shield_layers.increase_max(bonuses.shield_layers);
                dodge_chance += bonuses.dodge_chance;
                jamming_nullifying_chance += bonuses.jamming_nullifying_chance;
            });
        // Cap dodge chances and Jammin nullyfing resistance chances
        dodge_chance = max(dodge_chance, DODGE_CHANCE_CAP);
        jamming_nullifying_chance = max(jamming_nullifying_chance, JAMMING_NULLIFYING_CHANCE_CAP);

        // split powerups into active and passive
        let (active_powerups, passive_powerups): (Vec<_>, Vec<_>) = powerups
            .into_iter()
            .partition(|powerup| powerup.is_active());

        let active_powerups = active_powerups
            .into_iter()
            .map(|powerup| ActivePowerup::new(powerup))
            .collect::<Vec<_>>();

        let passive_powerups = passive_powerups
            .into_iter()
            .map(|powerup| PassivePowerup::new(powerup))
            .collect::<Vec<_>>();

        Self {
            name: spaceship.name.to_string(),
            hull_hitpoints,
            shield_layers,
            dodge_chance,
            jamming_nullifying_chance,
            active_powerups,
            passive_powerups,
            recent_hull_damage_per_turn: vec![0, 0, 0, 0, 0],
        }
    }

    // Maintenance operation to be carried each turn for the game engine
    pub fn end_of_turn_internals(&mut self) {
        // advance recent hull damage
        self.recent_hull_damage_per_turn.pop();
        self.recent_hull_damage_per_turn.insert(0, 0);
    }

    // A spaceship is defeated when his Hull HP reaches 0
    pub fn is_defeated(&self) -> bool {
        self.hull_hitpoints.depleted()
    }

    pub fn recent_hull_damage(&self) -> u8 {
        self.recent_hull_damage_per_turn.iter().sum()
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
            WeaponType::Projectile => self.apply_hull_damage(damage),
            WeaponType::Missile => self.apply_hull_damage(damage),
            WeaponType::Laser => {
                if self.shield_layers.depleted() {
                    self.apply_hull_damage(damage)
                } else {
                    self.deplete_shield_layer();
                }
            }
            WeaponType::Beam => {
                // only inflicts damage if shields are down
                if self.shield_layers.depleted() {
                    self.apply_hull_damage(damage)
                }
            }
        }
    }

    fn apply_hull_damage(&mut self, damage: u8) {
        self.hull_hitpoints.deplete(damage);
        if let Some(last) = self.recent_hull_damage_per_turn.last_mut() {
            *last += damage;
        }
    }

    fn deplete_shield_layer(&mut self) {
        self.shield_layers.deplete(1);
    }
}

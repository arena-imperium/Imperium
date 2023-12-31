use {
    super::{ConcretePowerup, FightOutcome, PowerUp},
    crate::{
        state::{HitPoints, RepairTarget, Shots, SpaceShip, WeaponType},
        utils::RandomNumberGenerator,
        BASE_DODGE_CHANCE, BASE_HULL_HITPOINTS, BASE_JAMMING_NULLIFYING_CHANCE, BASE_JAM_CHANCE,
        BASE_SHIELD_LAYERS, DODGE_CHANCE_CAP, JAMMING_NULLIFYING_CHANCE_CAP,
    },
    std::cmp::max,
};

// Note: Recently == 5 last turns
pub struct SpaceShipBattleCard {
    pub name: String,
    pub id: u64,
    // stats ----------------------------------------
    // Hitpoints
    pub hull_hitpoints: HitPoints,
    pub shield_layers: HitPoints,
    // chance to avoid attacks/jams
    pub dodge_chance: u8,
    pub jamming_nullifying_chance: u8,
    // powerups -------------------------------------
    pub concrete_powerups: Vec<ConcretePowerup>,
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

        let concrete_powerups: Vec<ConcretePowerup> = powerups
            .into_iter()
            .map(|powerup| ConcretePowerup::new(powerup))
            .collect();

        Self {
            name: spaceship.name.to_string(),
            id: spaceship.id,
            hull_hitpoints,
            shield_layers,
            dodge_chance,
            jamming_nullifying_chance,
            concrete_powerups,
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

    // return a MUTABLE iterator over the active powerups. You can then use this iterator to modify the active powerups
    fn get_active_powerups_mutable_iterator(
        &mut self,
    ) -> impl Iterator<Item = &mut ConcretePowerup> {
        self.concrete_powerups
            .iter_mut()
            .filter(|p| p.og_powerup.is_active())
    }

    pub fn fire_at(
        &mut self,
        target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
        damage: u8,
        shots: Shots,
        weapon_type: WeaponType,
        event_callback: &mut dyn FnMut(BattleEvent),
    ) {
        #[cfg(any(test, feature = "render-hooks"))]
        event_callback(BattleEvent::Fire {
            origin_id: self.id,
            target_id: target.id,
            damage,
            weapon_type,
            shots,
        });

        // Dodge roll
        match weapon_type {
            WeaponType::Plasma | WeaponType::Missile => { /* attacks cannot be dodged */ }
            _ => {
                let hit_roll = rng.roll_dice(100);
                let did_hit = hit_roll >= target.dodge_chance as u64;
                if !did_hit {
                    #[cfg(any(test, feature = "render-hooks"))]
                    event_callback(BattleEvent::Dodge { origin_id: self.id });
                    return;
                }
            }
        }

        match shots {
            Shots::Single | Shots::Salvo(1) => {
                target.apply_damage(damage, weapon_type, event_callback)
            }
            Shots::Salvo(shots) => {
                for _ in 0..shots {
                    target.apply_damage(damage, weapon_type, event_callback);
                }
            }
        };
    }

    pub fn jam(
        &mut self,
        target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
        charge_burn: u8,
        event_callback: &mut dyn FnMut(BattleEvent),
    ) {
        let jam_chance = BASE_JAM_CHANCE.saturating_sub(target.jamming_nullifying_chance);
        #[cfg(any(test, feature = "render-hooks"))]
        event_callback(BattleEvent::Jam {
            origin_id: self.id,
            target_id: target.id,
            chance: jam_chance,
            charge_burn,
        });
        if rng.roll_dice(BASE_JAM_CHANCE as usize) <= jam_chance as u64 {
            let target_id = target.id;
            let active_powerups_iter_mut = target.get_active_powerups_mutable_iterator();
            // filter powerups to get the active one, with some accumulated_charge
            let mut active_powerups_iter_mut_with_charge = active_powerups_iter_mut
                .filter(|a| a.accumulated_charge != 0)
                .peekable();

            // if there is no active powerup with charge, we can't jam anything and abort
            if active_powerups_iter_mut_with_charge.peek().is_none() {
                #[cfg(any(test, feature = "render-hooks"))]
                event_callback(BattleEvent::NothingToJam {
                    origin_id: self.id,
                    target_id,
                });
                return;
            }

            // now we know there is at least one active powerup with charge, we can pick one at random
            // collect all element in a vector
            let mut active_powerups_with_charge: Vec<&mut ConcretePowerup> =
                active_powerups_iter_mut_with_charge.collect();
            let random_index = rng.roll_dice(active_powerups_with_charge.len()) as usize - 1;
            #[cfg(any(test, feature = "render-hooks"))]
            {
                let target_powerup_name = active_powerups_with_charge[random_index].name.clone();
                event_callback(BattleEvent::ActivePowerUpJammed {
                    origin_id: self.id,
                    target_id,
                    active_power_up_name: target_powerup_name.to_string(),
                    active_power_up_index: random_index,
                    charge_burn,
                });
            }
            active_powerups_with_charge[random_index].accumulated_charge =
                active_powerups_with_charge[random_index]
                    .accumulated_charge
                    .saturating_sub(charge_burn);
        } else {
            #[cfg(any(test, feature = "render-hooks"))]
            event_callback(BattleEvent::JamResisted { origin_id: self.id });
        }
    }

    pub fn apply_damage(
        &mut self,
        damage: u8,
        weapon_type: WeaponType,
        event_callback: &mut dyn FnMut(BattleEvent),
    ) {
        match weapon_type {
            WeaponType::Projectile => self.apply_hull_damage(damage, event_callback),
            WeaponType::Missile => self.apply_hull_damage(damage, event_callback),
            WeaponType::Laser => {
                if self.shield_layers.depleted() {
                    self.apply_hull_damage(damage, event_callback)
                } else {
                    self.deplete_shield_layer(event_callback);
                }
            }
            WeaponType::Plasma => {
                // only inflicts damage if shields are down
                if self.shield_layers.depleted() {
                    self.apply_hull_damage(damage, event_callback)
                } else {
                    #[cfg(any(test, feature = "render-hooks"))]
                    event_callback(BattleEvent::ShieldCounterPlasmaAttack { origin_id: self.id });
                }
            }
        }
    }

    fn apply_hull_damage(&mut self, damage: u8, event_callback: &mut dyn FnMut(BattleEvent)) {
        #[cfg(any(test, feature = "render-hooks"))]
        event_callback(BattleEvent::HullDamaged {
            origin_id: self.id,
            damage,
        });
        self.hull_hitpoints.deplete(damage);
        if let Some(last) = self.recent_hull_damage_per_turn.first_mut() {
            *last += damage;
        }
    }

    fn deplete_shield_layer(&mut self, event_callback: &mut dyn FnMut(BattleEvent)) {
        #[cfg(any(test, feature = "render-hooks"))]
        event_callback(BattleEvent::ShieldLayerDown { origin_id: self.id });
        self.shield_layers.deplete(1);
    }
}

#[cfg(any(test, feature = "render-hooks"))]
pub enum BattleEvent {
    MatchStarted {},
    TurnStart {
        turn: u16,
    },
    MatchEnded {
        outcome: FightOutcome,
    },
    Fire {
        origin_id: u64,
        target_id: u64,
        damage: u8,
        weapon_type: WeaponType,
        shots: Shots,
    },
    // the sbc dodged the attack
    Dodge {
        origin_id: u64,
    },
    // the shield fully countered the plasma attack
    ShieldCounterPlasmaAttack {
        origin_id: u64,
    },
    HullDamaged {
        origin_id: u64,
        damage: u8,
    },
    ShieldLayerDown {
        origin_id: u64,
    },
    Jam {
        origin_id: u64,
        target_id: u64,
        chance: u8,
        charge_burn: u8,
    },
    JamResisted {
        origin_id: u64,
    },
    NothingToJam {
        origin_id: u64,
        target_id: u64,
    },
    ActivePowerUpJammed {
        origin_id: u64,
        target_id: u64,
        active_power_up_name: String,
        active_power_up_index: usize,
        charge_burn: u8,
    },
    Repair {
        origin_id: u64,
        repair_target: RepairTarget,
        amount: u8,
    },
}
#[cfg(not(any(test, feature = "render-hooks")))]
pub enum BattleEvent {}

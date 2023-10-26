use {
    super::{BattleEvent, ConcretePowerup, Effect, SpaceShipBattleCard},
    crate::{
        instructions::user_facing::Faction,
        state::{RepairTarget, SpaceShip},
        utils::RandomNumberGenerator,
        CHARGE_PER_TURN, CURRENCY_REWARD_FOR_ARENA_LOOSER, CURRENCY_REWARD_FOR_ARENA_WINNER,
        HEAT_DISSIPATION_PER_TURN,
    },
    anchor_lang::prelude::*,
};

pub struct FightEngine {
    event_callback: Box<dyn FnMut(BattleEvent)>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum FightOutcome {
    UserWon,
    OpponentWon,
    Draw,
}

impl FightEngine {
    pub fn new(event_callback: Box<dyn FnMut(BattleEvent)>) -> Self {
        Self { event_callback }
    }

    // This function is used to distribute currency to the winner and looser of an arena match.
    pub fn distribute_arena_currency(
        spaceship: &mut SpaceShip,
        opponent_spaceship: &mut SpaceShip,
        faction: Faction,
        outcome: FightOutcome,
    ) -> Result<()> {
        let currency = faction.legal_tender();

        match outcome {
            FightOutcome::UserWon => {
                spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_WINNER, currency)?;
                opponent_spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_LOOSER, currency)?;
            }
            FightOutcome::OpponentWon => {
                spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_LOOSER, currency)?;
                opponent_spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_WINNER, currency)?;
            }
            FightOutcome::Draw => {
                spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_LOOSER, currency)?;
                opponent_spaceship
                    .wallet
                    .credit(CURRENCY_REWARD_FOR_ARENA_LOOSER, currency)?;
            }
        }
        Ok(())
    }

    // Return true if the spaceship won the fight against opponent_spaceship
    pub fn fight(
        &mut self,
        mut s: &mut SpaceShipBattleCard,
        mut os: &mut SpaceShipBattleCard,
        fight_seed: u32,
        max_turns: u16,
    ) -> FightOutcome {
        let mut rng = RandomNumberGenerator::new(fight_seed as u64);

        #[cfg(any(test, feature = "render-hooks"))]
        (self.event_callback)(BattleEvent::MatchStarted {});

        let mut turn = 0;
        // will iterate until one of the spaceship is defeated or MATCH_MAX_TURN is reached
        while turn < max_turns {
            #[cfg(any(test, feature = "render-hooks"))]
            (self.event_callback)(BattleEvent::TurnStart { turn });

            // stopping condition, a player or both are defeated
            if s.is_defeated() || os.is_defeated() {
                break;
            }

            // these vectors will collect all effect that are to be applied to each spaceship
            // said effects come from active and passive modules
            let mut s_effects_to_apply = Vec::new();
            let mut os_effects_to_apply = Vec::new();

            let collect_effects =
                |powerups: &mut Vec<ConcretePowerup>,
                 effects_to_apply: &mut Vec<(Effect, usize)>| {
                    for (i, p) in powerups.iter_mut().enumerate() {
                        match p.is_active() {
                            true => {
                                if p.charge_and_activate(CHARGE_PER_TURN) {
                                    effects_to_apply.push((p.effect.clone(), i));
                                }
                            }
                            false => {
                                p.dissipate_heat(HEAT_DISSIPATION_PER_TURN);
                                if p.is_off_cooldown() {
                                    effects_to_apply.push((p.effect.clone(), i));
                                }
                            }
                        }
                    }
                };
            collect_effects(&mut s.concrete_powerups, &mut s_effects_to_apply);
            collect_effects(&mut os.concrete_powerups, &mut os_effects_to_apply);
            // TODO: might want to add some random shuffling of all action for more balance later on (create a (effect, emittor, target) array and shuffle it)
            self.apply_effects(s_effects_to_apply, &mut s, &mut os, &mut rng);
            self.apply_effects(os_effects_to_apply, &mut os, &mut s, &mut rng);

            s.end_of_turn_internals();
            os.end_of_turn_internals();

            // advance turn
            turn += 1;
        }

        // define fight outcome
        let outcome = match (s.is_defeated(), os.is_defeated()) {
            (true, false) => FightOutcome::OpponentWon,
            (false, true) => FightOutcome::UserWon,
            _ => FightOutcome::Draw,
        };

        #[cfg(any(test, feature = "render-hooks"))]
        (self.event_callback)(BattleEvent::MatchEnded { outcome });
        outcome
    }

    fn apply_effects(
        &mut self,
        effects: Vec<(Effect, usize)>,
        s_origin: &mut SpaceShipBattleCard,
        s_target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
    ) {
        for (effect, source_powerup_index) in effects {
            self.apply_effect(effect, source_powerup_index, s_origin, s_target, rng);
        }
    }

    // apply an effect to a spaceship
    // return wether something happened or not (in case of chance based and conditionnal effects)
    fn apply_effect(
        &mut self,
        effect: Effect,
        source_powerup_index: usize,
        s_origin: &mut SpaceShipBattleCard,
        s_target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
    ) -> bool {
        // effect is considered 'triggered' if the proba or condition was met
        let effect_triggered = match effect {
            Effect::Fire {
                damage,
                shots,
                weapon_type,
            } => {
                s_origin.fire_at(
                    s_target,
                    rng,
                    damage,
                    shots,
                    weapon_type,
                    &mut self.event_callback,
                );
                true
            }
            Effect::Repair { target, amount } => {
                #[cfg(any(test, feature = "render-hooks"))]
                (self.event_callback)(BattleEvent::Repair {
                    origin_id: s_origin.id,
                    repair_target: target,
                    amount,
                });
                match target {
                    RepairTarget::Hull => s_origin.hull_hitpoints.resplenish(amount),
                    RepairTarget::Shield => s_origin.shield_layers.resplenish(amount),
                };
                true
            }
            Effect::Jam { charge_burn } => {
                s_origin.jam(s_target, rng, charge_burn, &mut self.event_callback);
                true
            }
            Effect::Chance {
                probability,
                effect,
            } => {
                let roll = rng.roll_dice(100) as u8;
                if roll <= probability {
                    self.apply_effect(*effect, source_powerup_index, s_origin, s_target, rng);
                    true
                } else {
                    false
                }
            }
            Effect::Composite {
                effect1,
                effect2,
                probability1,
                probability2,
            } => {
                let total_probabilities = probability1 + probability2;
                let roll = rng.roll_dice(total_probabilities as usize) as u8;
                if roll <= probability1 {
                    self.apply_effect(*effect1, source_powerup_index, s_origin, s_target, rng);
                } else {
                    self.apply_effect(*effect2, source_powerup_index, s_origin, s_target, rng);
                }
                true
            }
            Effect::Conditionnal { condition, effect } => {
                if (condition.func)(s_origin) {
                    self.apply_effect(*effect, source_powerup_index, s_origin, s_target, rng)
                } else {
                    false
                }
            }
        };

        // if it's a passive and it was triggered, heat it (put it in cooldown period)
        if effect_triggered {
            let source_powerup = &mut s_origin.concrete_powerups[source_powerup_index];
            if !source_powerup.is_active() {
                source_powerup.heat();
            }
        }
        effect_triggered
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            engine::{PowerUp, LT_MODULES_COMMON, LT_MODULES_RARE, LT_MODULES_UNCOMMON},
            instructions::print_event,
            state::{mock_spaceship, ModuleClass},
            utils::LimitedString,
            MATCH_MAX_TURN,
        },
    };

    #[test]
    fn test_fight_empty_spaceships_should_draw() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {})); // event| print_event(event)
        let spaceship = mock_spaceship(vec![], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let outcome = fight_engine.fight(&mut s, &mut os, fight_seed, MATCH_MAX_TURN);

        assert!(matches!(outcome, FightOutcome::Draw));
    }

    #[test]
    fn test_plasma_damage_nullified_by_shield() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let slicer_module = LT_MODULES_COMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Slicer"))
            .unwrap();
        let capacitative_shield_battery_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Capacitative Shield Battery"))
            .unwrap();
        let spaceship = mock_spaceship(vec![slicer_module], vec![], vec![]);
        let opponent_spaceship =
            mock_spaceship(vec![capacitative_shield_battery_module], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 18;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        assert_eq!(os.shield_layers.current, os.shield_layers.max);
        assert_eq!(os.hull_hitpoints.current, os.hull_hitpoints.max);
    }

    #[test]
    fn test_laser_blocked_by_shield() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let pulse_laser_module = LT_MODULES_COMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Pulse Laser"))
            .unwrap();
        let capacitative_shield_battery_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Capacitative Shield Battery"))
            .unwrap();
        let spaceship = mock_spaceship(vec![pulse_laser_module], vec![], vec![]);
        let opponent_spaceship =
            mock_spaceship(vec![capacitative_shield_battery_module], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 10;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        assert_eq!(os.shield_layers.current, os.shield_layers.max - 1);
        assert_eq!(os.hull_hitpoints.current, os.hull_hitpoints.max);
    }

    #[test]
    fn test_fight_missile_weapon() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let module = LT_MODULES_COMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Light Missile Launcher I"))
            .unwrap();
        let spaceship = mock_spaceship(vec![module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let outcome = fight_engine.fight(&mut s, &mut os, fight_seed, MATCH_MAX_TURN);

        assert!(matches!(outcome, FightOutcome::UserWon));
    }

    #[test]
    fn test_fight_laser_weapon() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Heavy Pulse Laser"))
            .unwrap();
        let spaceship = mock_spaceship(vec![module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let outcome = fight_engine.fight(&mut s, &mut os, fight_seed, MATCH_MAX_TURN);

        assert!(matches!(outcome, FightOutcome::UserWon));
    }

    #[test]
    fn test_fight_projectile_weapon() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("125mm Dual Autocannon"))
            .unwrap();
        let spaceship = mock_spaceship(vec![module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let outcome = fight_engine.fight(&mut s, &mut os, fight_seed, MATCH_MAX_TURN);

        assert!(matches!(outcome, FightOutcome::UserWon));
    }

    #[test]
    fn test_fight_capacitative_shield_repair() {
        let mut fight_engine = FightEngine::new(Box::new(|_| {}));
        let capacitative_shield_battery_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Capacitative Shield Battery"))
            .unwrap();
        let heavy_pulse_laser_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Heavy Pulse Laser"))
            .unwrap();
        let spaceship = mock_spaceship(vec![capacitative_shield_battery_module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(
            vec![
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module,
            ],
            vec![],
            vec![],
        );
        let fight_seed = 4;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 17;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        assert_eq!(s.shield_layers.current, s.shield_layers.max);
    }

    #[test]
    fn test_fight_capacitative_hull_repair() {
        let mut fight_engine = FightEngine::new(Box::new(|e| print_event(e)));
        let capacitative_shield_battery_module = LT_MODULES_RARE
            .into_iter()
            .find(|m| m.name == LimitedString::new("Capacitative Armor"))
            .unwrap();
        let heavy_pulse_laser_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Heavy Pulse Laser"))
            .unwrap();
        let spaceship = mock_spaceship(vec![capacitative_shield_battery_module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(
            vec![
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module.clone(),
                heavy_pulse_laser_module,
            ],
            vec![],
            vec![],
        );
        let fight_seed = 4;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 18;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        // minus damages + heal
        assert_eq!(s.hull_hitpoints.current, s.hull_hitpoints.max - 8 + 2);
    }

    #[test]
    fn test_fight_jam_module_resisted() {
        let mut fight_engine = FightEngine::new(Box::new(|e| print_event(e)));
        let phantom_burst_jammer_module = LT_MODULES_RARE
            .into_iter()
            .find(|m| m.name == LimitedString::new("'Phantom' Burst Jammer"))
            .unwrap();
        let heavy_pulse_laser_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Heavy Pulse Laser"))
            .unwrap();
        let phantom_burst_jammer_module_charge_burn =
            if let ModuleClass::Jammer(_, jms) = phantom_burst_jammer_module.class {
                jms.charge_burn.clone()
            } else {
                panic!("wrong module class")
            };
        let heavy_pulse_laser_charge_time = heavy_pulse_laser_module.get_charge_time().unwrap();
        let spaceship = mock_spaceship(vec![phantom_burst_jammer_module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![heavy_pulse_laser_module], vec![], vec![]);
        let fight_seed = 1;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 15;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        assert_ne!(
            s.concrete_powerups.first().unwrap().accumulated_charge,
            heavy_pulse_laser_charge_time - phantom_burst_jammer_module_charge_burn
        );
    }

    #[test]
    fn test_fight_jam_module() {
        let mut fight_engine = FightEngine::new(Box::new(|e| print_event(e)));
        let phantom_burst_jammer_module = LT_MODULES_RARE
            .into_iter()
            .find(|m| m.name == LimitedString::new("'Phantom' Burst Jammer"))
            .unwrap();
        let heavy_pulse_laser_module = LT_MODULES_UNCOMMON
            .into_iter()
            .find(|m| m.name == LimitedString::new("Heavy Pulse Laser"))
            .unwrap();
        let phantom_burst_jammer_module_charge_burn =
            if let ModuleClass::Jammer(_, jms) = phantom_burst_jammer_module.class {
                jms.charge_burn.clone()
            } else {
                panic!("wrong module class")
            };
        let heavy_pulse_laser_charge_time = heavy_pulse_laser_module.get_charge_time().unwrap();
        let spaceship = mock_spaceship(vec![phantom_burst_jammer_module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![heavy_pulse_laser_module], vec![], vec![]);
        let fight_seed = 2;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = 15;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);

        // -1 cause we are still at turn 15 here
        assert_eq!(
            os.concrete_powerups.first().unwrap().accumulated_charge,
            heavy_pulse_laser_charge_time - 1 - phantom_burst_jammer_module_charge_burn
        );
    }

    #[test]
    fn test_fight_jam_module_nothing_to_jam() {
        let mut fight_engine = FightEngine::new(Box::new(|e| print_event(e)));
        let phantom_burst_jammer_module = LT_MODULES_RARE
            .into_iter()
            .find(|m| m.name == LimitedString::new("'Phantom' Burst Jammer"))
            .unwrap();
        let spaceship = mock_spaceship(vec![phantom_burst_jammer_module], vec![], vec![]);
        let opponent_spaceship = mock_spaceship(vec![], vec![], vec![]);
        let fight_seed = 2;

        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);
        let turns = MATCH_MAX_TURN;
        let _ = fight_engine.fight(&mut s, &mut os, fight_seed, turns);
    }
}

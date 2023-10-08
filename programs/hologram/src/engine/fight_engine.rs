use {
    super::{ActivePowerup, BattleEvent, Effect, SpaceShipBattleCard},
    crate::{
        instructions::user_facing::Faction,
        state::{RepairTarget, SpaceShip},
        utils::RandomNumberGenerator,
        CHARGE_PER_TURN, CURRENCY_REWARD_FOR_ARENA_LOOSER, CURRENCY_REWARD_FOR_ARENA_WINNER,
        MATCH_MAX_TURN,
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
        spaceship: &SpaceShip,
        opponent_spaceship: &SpaceShip,
        fight_seed: u32,
    ) -> FightOutcome {
        let mut rng = RandomNumberGenerator::new(fight_seed as u64);

        // generate SpaceShipBattleCards, another data-representation of a SpaceShip object optimized for battle
        let mut s = SpaceShipBattleCard::new(&spaceship);
        let mut os = SpaceShipBattleCard::new(&opponent_spaceship);

        let mut turn = 0;
        while turn < MATCH_MAX_TURN {
            #[cfg(feature = "render-hooks")]
            (self.event_callback)(BattleEvent::TurnStart { turn });
            if s.is_defeated() || os.is_defeated() {
                break;
            }

            // charge active modules of the spaceship and keep track of the ones reaching activation treshold
            let mut s_effects_to_apply = Vec::new();
            let mut os_effects_to_apply = Vec::new();
            let collect_effects =
                |powerups: &mut Vec<ActivePowerup>, effects_to_apply: &mut Vec<Effect>| {
                    for a in powerups.iter_mut() {
                        if a.charge_and_activate(CHARGE_PER_TURN) {
                            effects_to_apply.extend(a.effects.clone());
                        }
                    }
                };
            collect_effects(&mut s.active_powerups, &mut s_effects_to_apply);
            collect_effects(&mut os.active_powerups, &mut os_effects_to_apply);
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

        #[cfg(feature = "render-hooks")]
        (self.event_callback)(BattleEvent::MatchEnded { outcome });
        outcome
    }

    fn apply_effects(
        &mut self,
        effects: Vec<Effect>,
        s_origin: &mut SpaceShipBattleCard,
        s_target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
    ) {
        for effect in effects {
            self.apply_effect(effect, s_origin, s_target, rng);
        }
    }

    fn apply_effect(
        &mut self,
        effect: Effect,
        s_origin: &mut SpaceShipBattleCard,
        s_target: &mut SpaceShipBattleCard,
        rng: &mut RandomNumberGenerator,
    ) {
        match effect {
            Effect::Fire {
                damage,
                shots,
                weapon_type,
            } => s_origin.fire_at(
                s_target,
                rng,
                damage,
                shots,
                weapon_type,
                &mut self.event_callback,
            ),
            Effect::Repair { target, amount } => match target {
                RepairTarget::Hull => s_origin.hull_hitpoints.resplenish(amount),
                RepairTarget::Shield => s_origin.shield_layers.resplenish(amount),
            },
            Effect::Jam {
                chance,
                charge_burn,
            } => {
                s_origin.jam(s_target, rng, chance, charge_burn, &mut self.event_callback);
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
                    self.apply_effect(*effect1, s_origin, s_target, rng);
                } else {
                    self.apply_effect(*effect2, s_origin, s_target, rng);
                }
            }
            Effect::Conditionnal { condition, effect } => todo!(),
            Effect::DamageAbsorbtion {
                weapon_type,
                chance,
            } => todo!(),
        }
    }
}

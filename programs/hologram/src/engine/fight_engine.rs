use {
    super::{ActiveEffect, ActivePowerup, PassivePowerup, PowerUp, SpaceShipBattleCard},
    crate::{
        instructions::user_facing::Faction,
        state::{RepairTarget, SpaceShip},
        utils::RandomNumberGenerator,
        CURRENCY_REWARD_FOR_ARENA_WINNER,
    },
    anchor_lang::prelude::*,
};

pub const MATCH_MAX_TURN: u16 = 1000;
pub const CHARGE_PER_TURN: u8 = 1;

pub struct FightEngine {}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum FightOutcome {
    UserWon,
    OpponentWon,
    Draw,
}

impl FightEngine {
    // This function is used to distribute experience points to the winner and loser of an arena match.
    // The winner gains experience points equal to the maximum of 1 and the difference between the loser's level and their own.
    pub fn distribute_arena_experience(winner: &mut SpaceShip, looser: &SpaceShip) -> Result<()> {
        let winner_lvl = winner.experience.current_level;
        let looser_lvl = looser.experience.current_level;

        // Winning in the Arena will grant you max(1, opponent_spaceship_level - spaceship_level) XP points
        let xp_gain = std::cmp::max(1, looser_lvl + winner_lvl);
        winner.gain_experience(xp_gain)
    }

    // This function is used to distribute currency to the winner of an arena match.
    pub fn distribute_arena_currency(winner: &mut SpaceShip, faction: Faction) -> Result<()> {
        let currency = faction.legal_tender();
        winner
            .wallet
            .credit(CURRENCY_REWARD_FOR_ARENA_WINNER as u16, currency)
    }

    // Return true if the spaceship won the fight against opponent_spaceship
    pub fn fight<'a>(
        spaceship: &SpaceShip,
        opponent_spaceship: &SpaceShip,
        fight_seed: u32,
    ) -> FightOutcome {
        let mut rng = RandomNumberGenerator::new(fight_seed as u64);

        // generate SpaceShipBattleCards, a new object that contains the spaceship's stats ready for the battle
        let mut s = Self::generate_spaceship_battlecard(&spaceship);
        let mut os = Self::generate_spaceship_battlecard(&opponent_spaceship);

        #[cfg(not(any(test, feature = "testing")))]
        msg!("s actives: {:?}", s.active_powerups.iter().map(|a| a.name));

        #[cfg(not(any(test, feature = "testing")))]
        msg!(
            "os actives: {:?}",
            os.active_powerups.iter().map(|a| a.name)
        );

        // First loop through each player powerups, find all PassivePowerups, and apply them to the battlecards
        // @TODO

        // Second loop through remaining powerups, find all ActivePowerups, and add them to the battlecard
        let mut turn = 0;
        while turn < MATCH_MAX_TURN {
            #[cfg(not(any(test, feature = "testing")))]
            msg!("turn: {} ----------------", turn);
            if s.is_defeated() || os.is_defeated() {
                break;
            }

            // charge active modules of the spaceship and keep track of the ones reaching activation treshold
            let mut s_effects_to_apply = Vec::new();
            let mut os_effects_to_apply = Vec::new();
            s.active_powerups.iter_mut().for_each(|a| {
                // increase charge and attempt activation
                if a.charge_and_activate(CHARGE_PER_TURN) {
                    // module charge reached the treshold, collect effects
                    a.effects
                        .iter()
                        .for_each(|e| s_effects_to_apply.push(e.clone()));
                }
            });
            os.active_powerups.iter_mut().for_each(|a| {
                // increase charge and attempt activation
                if a.charge_and_activate(CHARGE_PER_TURN) {
                    // module charge reached the treshold, collect effects
                    a.effects
                        .iter()
                        .for_each(|e| os_effects_to_apply.push(e.clone()));
                }
            });

            apply_effects(s_effects_to_apply, &mut s, &mut os, &mut rng);
            apply_effects(os_effects_to_apply, &mut os, &mut s, &mut rng);

            // advance turn
            turn += 1;
        }

        // define fight outcome
        match (s.is_defeated(), os.is_defeated()) {
            (true, false) => FightOutcome::OpponentWon,
            (false, true) => FightOutcome::UserWon,
            _ => FightOutcome::Draw,
        }
    }

    fn generate_spaceship_battlecard(spaceship: &SpaceShip) -> SpaceShipBattleCard {
        // convert all modules, drones, mutations to PowerUp
        let mut powerups = Vec::new();
        for module in &spaceship.modules {
            powerups.push(Box::new(module.clone()) as Box<dyn PowerUp>);
        }
        for drone in &spaceship.drones {
            powerups.push(Box::new(drone.clone()) as Box<dyn PowerUp>);
        }
        for mutation in &spaceship.mutations {
            powerups.push(Box::new(mutation.clone()) as Box<dyn PowerUp>);
        }

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

        // add powerups to the spaceship battlecard
        let battlecard = SpaceShipBattleCard {
            name: spaceship.name.to_string(),
            hull_hitpoints: spaceship.get_hull_hitpoints(),
            shield_layers: spaceship.get_shield_layers(),
            dodge_chance: spaceship.get_dodge_chance(),
            jamming_nullifying_chance: spaceship.get_jamming_nullifying_chance(),
            active_powerups,
            passive_powerups,
        };

        battlecard
    }
}

fn apply_effects(
    effects: Vec<ActiveEffect>,
    s_origin: &mut SpaceShipBattleCard,
    s_target: &mut SpaceShipBattleCard,
    rng: &mut RandomNumberGenerator,
) {
    for effect in effects {
        apply_effect(effect, s_origin, s_target, rng);
    }
}

fn apply_effect(
    effect: ActiveEffect,
    s_origin: &mut SpaceShipBattleCard,
    s_target: &mut SpaceShipBattleCard,
    rng: &mut RandomNumberGenerator,
) {
    match effect {
        ActiveEffect::Fire {
            damage,
            shots,
            weapon_type,
        } => s_origin.fire_at(s_target, rng, damage, shots, weapon_type),
        ActiveEffect::Repair { target, amount } => match target {
            RepairTarget::Hull => s_origin.hull_hitpoints.resplenish(amount),
            RepairTarget::Shield => s_origin.shield_layers.resplenish(amount),
        },
        ActiveEffect::Jam {
            chance,
            charge_burn,
        } => {
            s_origin.jam(s_target, rng, chance, charge_burn);
        }
        ActiveEffect::Composite {
            effect1,
            effect2,
            probability1,
            probability2,
        } => {
            let total_probabilities = probability1 + probability2;
            let roll = rng.roll_dice(total_probabilities as usize) as u8;
            if roll <= probability1 {
                apply_effect(*effect1, s_origin, s_target, rng);
            } else {
                apply_effect(*effect2, s_origin, s_target, rng);
            }
        }
    }
}

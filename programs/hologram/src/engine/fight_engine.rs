use {
    super::{ActiveEffect, ActivePowerup, PassivePowerup, PowerUp, SpaceShipBattleCard},
    crate::{
        instructions::user_facing::Faction, state::SpaceShip, utils::RandomNumberGenerator,
        BASE_JAM_CHANCE, CURRENCY_REWARD_FOR_ARENA_WINNER,
    },
    anchor_lang::prelude::*,
};

pub const CHARGE_PER_TURN: u8 = 10;
pub const MATCH_MAX_TURN: u8 = 100;

pub struct FightEngine {}

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
    ) -> bool {
        let mut rng = RandomNumberGenerator::new(fight_seed as u64);

        // generate SpaceShipBattleCards, a new object that contains the spaceship's stats ready for the battle
        let spaceship_bc = Self::generate_spaceship_battlecard(&spaceship);
        let opponent_spaceship_bc = Self::generate_spaceship_battlecard(&opponent_spaceship);

        // First loop through each player powerups, find all PassivePowerups, and apply them to the battlecard

        // Second loop through remaining powerups, find all ActivePowerups, and add them to the battlecard

        // Third, iterate through each turn. Highest celerity battlecard starts, then the other one, and so on.
        // Each turn duration is 10 unit of time, add that to all ActivePowerups and fire the ones that are ready
        // apply effects, in order:
        // - temporary maluses/bonuses
        // - repairers
        // - damage dealers

        // Determine the starting spaceship based on their celerity
        let mut battlecards = if spaceship_bc.celerity > opponent_spaceship_bc.celerity {
            [spaceship_bc, opponent_spaceship_bc]
        } else {
            [opponent_spaceship_bc, spaceship_bc]
        };

        //
        let mut turn = 0;
        let winner: SpaceShipBattleCard;
        while turn < MATCH_MAX_TURN {
            // Player is alternatively the spaceship and the opponnent_spaceship
            let (spaceship_slice, opponent_slice) = battlecards.split_at_mut(1);
            // switch order for next turn
            let spaceship = &mut spaceship_slice[0];
            let opponent = &mut opponent_slice[0];

            if spaceship.is_defeated() {
                break;
            }

            // charge active modules of the spaceship and keep track of the ones reaching activation treshold
            let mut effects_to_apply = Vec::new();
            spaceship.active_powerups.iter_mut().for_each(|a| {
                // increase charge and attempt activation
                if a.charge_and_activate(CHARGE_PER_TURN) {
                    // module charge reached the treshold, collect effects
                    a.effects
                        .iter()
                        .for_each(|e| effects_to_apply.push(e.clone()));
                }
            });

            for effect in effects_to_apply {
                match effect {
                    ActiveEffect::Fire {
                        damages,
                        shots,
                        projectile_speed,
                    } => spaceship.fire_at(opponent, &mut rng, damages, shots, projectile_speed),
                    ActiveEffect::RepairHull { amount } => {
                        spaceship.hull_hitpoints.resplenish(amount);
                    }
                    ActiveEffect::RepairArmor { amount } => {
                        spaceship.armor_hitpoints.resplenish(amount);
                    }
                    ActiveEffect::RepairShield { amount } => {
                        spaceship.shield_hitpoints.resplenish(amount);
                    }
                    ActiveEffect::Jam { charge_burn } => {
                        let jam_chance = BASE_JAM_CHANCE - spaceship.jamming_nullifying_chance;
                        if rng.roll_dice(BASE_JAM_CHANCE as usize) <= jam_chance as u64 {
                            if let Some(target) = rng.choose_mut(&mut opponent.active_powerups) {
                                target.accumulated_charge -= charge_burn;
                            }
                        }
                    }
                    ActiveEffect::Composite {
                        effect1,
                        effect2,
                        probability1,
                        probability2,
                    } => {
                        // @TODO: Implement the logic for the Composite effect
                        // This is just a placeholder
                        println!(
                            "effect1: {:?}, effect2: {:?}, probability1: {}, probability2: {}",
                            effect1, effect2, probability1, probability2
                        );
                    }
                }
            }

            // advance turn
            turn += 1;
            // switch order for next turn
            battlecards.reverse();
        }

        // emulate game engine for now @TODO
        let winner_roll = rng.roll_dice(2);

        match winner_roll {
            1 => true,
            2 => false,
            _ => panic!("Invalid dice roll"),
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
        let mut battlecard = SpaceShipBattleCard {
            hull_hitpoints: spaceship.get_hull_hitpoints(),
            armor_hitpoints: spaceship.get_armor_hitpoints(),
            shield_hitpoints: spaceship.get_shield_hitpoints(),
            celerity: spaceship.get_celerity(),
            turret_charge_time_reduction: spaceship.get_turret_charge_time_reduction(),
            turret_bonus_projectile_speed: spaceship.get_turret_bonus_projectile_speed(),
            dodge_chance: spaceship.get_dodge_chance(),
            jamming_nullifying_chance: spaceship.get_jamming_nullifying_chance(),
            active_powerups,
            passive_powerups,
        };

        // Note: during the fight, we will rely on active modules stats for all calculation.
        // All the modifiers from stats and other passive modules are already compounded in

        battlecard
    }
}

use {
    crate::{
        instructions::user_facing::Faction, state::SpaceShip, utils::RandomNumberGenerator,
        CURRENCY_REWARD_FOR_ARENA_WINNER,
    },
    anchor_lang::prelude::*,
};

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

    pub fn fight<'a>(
        s1: &'a mut SpaceShip,
        s2: &'a mut SpaceShip,
        fight_seed: u32,
    ) -> (&'a mut SpaceShip, &'a mut SpaceShip) {
        let mut rng = RandomNumberGenerator::new(fight_seed as u64);
        // emulate game engine for now @TODO
        let winner_roll = rng.roll_dice(2);
        match winner_roll {
            1 => (s1, s2),
            2 => (s2, s1),
            _ => panic!("Invalid dice roll"),
        }
    }
}

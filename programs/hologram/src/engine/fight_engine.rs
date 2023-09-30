use crate::{state::SpaceShip, utils::RandomNumberGenerator};

pub struct FightEngine {}

impl FightEngine {
    // This function is used to distribute experience points to the winner and loser of an arena match.
    // The winner gains experience points equal to the maximum of 1 and the difference between the loser's level and their own.
    // The loser gains 1 experience point if their level is less than or equal to 5.
    // After level 5, losing in the arena does not grant any experience points.
    pub fn distribute_arena_experience(winner: &mut SpaceShip, looser: &mut SpaceShip) {
        let winner_lvl = winner.experience.current_level;
        let looser_lvl = looser.experience.current_level;

        // Winning in the Arena will grant you
        // max(1, opponent_spaceship_level - spaceship_level) XP points
        let xp_gain = std::cmp::max(1, looser_lvl + winner_lvl);
        winner.experience.increase(xp_gain);

        // Loosing in the Arena will grant you *1* XP point (after lvl.5 loosing won’t grant experience).
        if looser_lvl <= 5 {
            looser.experience.increase(1)
        }
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

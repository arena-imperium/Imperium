use {
    crate::{error::HologramError, XP_REQUIERED_PER_LEVEL_MULT},
    anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Experience {
    pub current_level: u8,
    pub current_exp: u16,
    pub exp_to_next_level: u16,
    pub available_stat_points: u8,
}

impl Experience {
    pub fn credit_stat_point(&mut self, amount: u8) {
        self.available_stat_points += amount;
    }

    pub fn debit_stat_point(&mut self, amount: u8) -> Result<()> {
        require!(
            self.available_stat_points >= amount,
            HologramError::InsufficientStatPoints
        );
        self.available_stat_points -= amount;
        Ok(())
    }

    // return the amount of experience needed to reach the next level
    // formula: next_level * XP_REQUIERED_PER_LEVEL_MULT, capped at MAX_XP_PER_LEVEL
    pub fn experience_to_next_level(&self) -> u16 {
        (self.current_level as u16 + 1) * XP_REQUIERED_PER_LEVEL_MULT as u16
    }
}

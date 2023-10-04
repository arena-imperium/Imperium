use {
    crate::{error::HologramError, XP_REQUIERED_PER_LEVEL_MULT},
    anchor_lang::prelude::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Experience {
    pub current_level: u8,
    pub current_exp: u16,
    pub exp_to_next_level: u16,
    pub available_subsystem_upgrade_points: u8,
}

impl Experience {
    pub fn credit_subsystem_upgrade_point(&mut self, amount: u8) {
        self.available_subsystem_upgrade_points += amount;
    }

    pub fn debit_subsystem_upgrade_point(&mut self, amount: u8) -> Result<()> {
        require!(
            self.available_subsystem_upgrade_points >= amount,
            HologramError::InsufficientSubsystemUpgradePoints
        );
        self.available_subsystem_upgrade_points -= amount;
        Ok(())
    }

    // return the amount of experience needed to reach the next level
    // formula: next_level * XP_REQUIERED_PER_LEVEL_MULT, capped at MAX_XP_PER_LEVEL
    pub fn experience_to_next_level(&self) -> u16 {
        (self.current_level as u16 + 1) * XP_REQUIERED_PER_LEVEL_MULT as u16
    }
}

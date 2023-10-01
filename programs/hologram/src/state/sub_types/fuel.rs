use {crate::error::HologramError, anchor_lang::prelude::*};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Fuel {
    pub max: u8,
    pub current: u8,
    // players can collect DAILY_FUEL_ALLOWANCE once per FUEL_ALLOWANCE_COOLDOWN period, this is the timestamp of their last collection
    pub daily_allowance_last_collection: i64,
}

impl Fuel {
    pub fn consume(&mut self, amount: u8) -> Result<()> {
        require!(self.current > amount, HologramError::InsufficientFuel);
        self.current -= amount;
        Ok(())
    }

    pub fn refill(&mut self, amount: u8) -> Result<()> {
        self.current = std::cmp::min(self.current + amount, self.max);
        Ok(())
    }
}

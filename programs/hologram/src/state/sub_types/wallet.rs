use {crate::error::HologramError, anchor_lang::prelude::*};

#[derive(Clone, Copy)]
pub enum Currency {
    ImperialCredit,
    ActivateNanitePaste,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Wallet {
    pub imperial_credits: u16,
    pub activate_nanite_paste: u16,
}

impl Wallet {
    pub fn get_balance(&self, currency: Currency) -> u16 {
        match currency {
            Currency::ImperialCredit => self.imperial_credits,
            Currency::ActivateNanitePaste => self.activate_nanite_paste,
        }
    }

    pub fn debit(&mut self, amount: u16, currency: Currency) -> Result<()> {
        match currency {
            Currency::ImperialCredit => {
                require!(
                    self.imperial_credits >= amount,
                    HologramError::InsufficientFunds
                );
                self.imperial_credits -= amount;
            }
            Currency::ActivateNanitePaste => {
                require!(
                    self.activate_nanite_paste >= amount,
                    HologramError::InsufficientFunds
                );
                self.activate_nanite_paste -= amount;
            }
        }
        Ok(())
    }

    pub fn credit(&mut self, amount: u8, currency: Currency) -> Result<()> {
        match currency {
            Currency::ImperialCredit => {
                self.imperial_credits = self
                    .imperial_credits
                    .checked_add(amount as u16)
                    .ok_or(HologramError::Overflow)?;
            }
            Currency::ActivateNanitePaste => {
                self.activate_nanite_paste = self
                    .activate_nanite_paste
                    .checked_add(amount as u16)
                    .ok_or(HologramError::Overflow)?;
            }
        };
        Ok(())
    }
}

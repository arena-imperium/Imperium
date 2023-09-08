//! Error types

use anchor_lang::prelude::*;

#[error_code]
pub enum HologramError {
    #[msg("Overflow in arithmetic operation")]
    MathOverflow,
    #[msg("Limited String can store 64 chars at most")]
    LimitedStringLengthExceeded,
    #[msg("The game state does not permit this action")]
    InvalidAction,
}

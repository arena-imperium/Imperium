//! Error types

use anchor_lang::prelude::*;

#[error_code]
pub enum HologramError {
    #[msg("Overflow in arithmetic operation")]
    MathOverflow,
    #[msg("Limited String can store 64 chars at most")]
    LimitedStringLengthExceeded,
    #[msg("There cannot be more than 25 spaceships per user account")]
    SpaceshipsLimitExceeded,
    #[msg("A spaceship with the same name already exists for this user account")]
    SpaceshipNameAlreadyExists,
    #[msg("The switchboard validation failed")]
    SwitchboardFunctionValidationFailed,
    #[msg("The switchboard randomness for the Spaceship has already been requested")]
    SpaceshipRandomnessAlreadyRequested,
    #[msg("The switchboard randomness for the Spaceship has already been settled")]
    SpaceshipRandomnessAlreadySettled,
    #[msg("The switchboard request was not successful")]
    SwitchboardRequestNotSuccessful,
    #[msg("The game state does not permit this action")]
    InvalidAction,
}

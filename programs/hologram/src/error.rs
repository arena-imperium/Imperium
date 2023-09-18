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
    #[msg("The switchboard function for the Spaceship randomness has already been requested")]
    SpaceshipRandomnessAlreadyRequested,
    #[msg("The switchboard randomness for the Spaceship has already been settled")]
    SpaceshipRandomnessAlreadySettled,
    #[msg("The switchboard function for the Arena Matchmaking has already been requested")]
    ArenaMatchmakingAlreadyRequested,
    #[msg("The switchboard request was not successful")]
    SwitchboardRequestNotSuccessful,
    #[msg("The spaceship doesn't have enough fuel for this action")]
    InsufficientFuel,
    #[msg("The spaceship is already queued for matchmaking")]
    MatchmakingAlreadyInQueue,
    #[msg("No matchmaking queue was found for this spaceship level")]
    MatchmakingQueueNotFound,
    #[msg("The matchmaking queue is full")]
    MatchmakingQueueFull,
    #[msg("The matchmaking queue cannot handle more requests at the moment. Please retry later")]
    MatchmakingTooManyRequests,
    #[msg("The game state does not permit this action")]
    InvalidAction,
}

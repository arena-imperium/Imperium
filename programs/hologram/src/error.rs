//! Error types

use anchor_lang::prelude::*;

#[error_code]
pub enum HologramError {
    #[msg("Overflow in arithmetic operation")]
    Overflow,
    #[msg("Limited String can store 64 chars at most")]
    LimitedStringLengthExceeded,
    #[msg("There cannot be more than 25 spaceships per user account")]
    SpaceshipsLimitExceeded,
    #[msg("A spaceship with the same name already exists for this user account")]
    SpaceshipNameAlreadyExists,
    #[msg("The switchboard validation failed")]
    SwitchboardFunctionValidationFailed,

    #[msg("The switchboard function for the randomness has already been requested")]
    SpaceshipRandomnessAlreadyRequested,
    #[msg("The switchboard function for the Arena Matchmaking has already been requested")]
    ArenaMatchmakingAlreadyRequested,
    #[msg("The switchboard function for the crate picking has already been requested")]
    CratePickingAlreadyRequested,
    #[msg("The switchboard function request for the randomness has already been settled")]
    SpaceshipRandomnessAlreadySettled,
    #[msg("The switchboard function request for the arena matchmaking has already been settled")]
    ArenaMatchmakingAlreadySettled,
    #[msg("The switchboard function request for crate picking has already been settled")]
    CratePickingAlreadySettled,

    #[msg("The switchboard request was not successful")]
    SwitchboardRequestNotSuccessful,
    #[msg("The spaceship doesn't have enough fuel for this action")]
    InsufficientFuel,
    #[msg("The spaceship is already queued for matchmaking")]
    ArenaMatchmakingAlreadyInQueue,
    #[msg("No matchmaking queue was found for this spaceship level")]
    MatchmakingQueueNotFound,
    #[msg("The matchmaking queue is full")]
    MatchmakingQueueFull,
    #[msg("The matchmaking queue cannot handle more requests at the moment. Please retry later")]
    MatchmakingTooManyRequests,
    #[msg("The spaceship must allocate his level up stats and powerup before being able to join the arena")]
    PendingStatOrPowerup,
    #[msg("The queue is currently empty, please retry later")] // concurrency issue
    NoSpaceshipsInQueue,
    #[msg("The spaceship fuel allowance is not available yet")]
    FuelAllowanceOnCooldown,
    #[msg("The spaceship has no available stats points to spend")]
    NoAvailableStatsPoints,
    #[msg("The spaceship has no available crate to pick at this moment")]
    NoCrateAvailable,
    #[msg("The game state does not permit this action")]
    InvalidAction,
}

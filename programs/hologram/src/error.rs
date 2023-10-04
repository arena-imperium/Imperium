//! Error types

use anchor_lang::prelude::*;

#[error_code]
pub enum HologramError {
    #[msg("Overflow in arithmetic operation")]
    Overflow = 0,
    #[msg("Limited String can store 64 chars at most")]
    LimitedStringLengthExceeded,
    #[msg("There cannot be more than 25 spaceships per user account")]
    SpaceshipsLimitExceeded,
    #[msg("A spaceship with the same name already exists for this user account")]
    SpaceshipNameAlreadyExists,
    #[msg("The switchboard validation failed")]
    SwitchboardFunctionValidationFailed,

    #[msg("The switchboard function for the randomness has already been requested")]
    SpaceshipRandomnessAlreadyRequested, // 5
    #[msg("The switchboard function for the Arena Matchmaking has already been requested")]
    ArenaMatchmakingAlreadyRequested,
    #[msg("The switchboard function for the crate picking has already been requested")]
    CratePickingAlreadyRequested,
    #[msg("The switchboard function request for the randomness has already been settled")]
    SpaceshipRandomnessAlreadySettled,
    #[msg("The switchboard function request for the arena matchmaking has already been settled")]
    ArenaMatchmakingAlreadySettled,
    #[msg("The switchboard function request for crate picking has already been settled")]
    CratePickingAlreadySettled, // 10

    #[msg("The switchboard request was not successful")]
    SwitchboardRequestNotSuccessful,
    #[msg("The spaceship doesn't have enough fuel for this action")]
    InsufficientFuel,
    #[msg("The spaceship is already queued for matchmaking")]
    ArenaMatchmakingAlreadyInQueue,
    #[msg("No matchmaking queue was found for this spaceship level")]
    MatchmakingQueueNotFound,
    #[msg("The matchmaking queue is full")]
    MatchmakingQueueFull, // 15
    #[msg("The matchmaking queue cannot handle more requests at the moment. Please retry later")]
    MatchmakingTooManyRequests,
    #[msg("The queue is currently empty, please retry later")] // concurrency issue
    NoSpaceshipsInQueue,
    #[msg("The spaceship fuel allowance is not available yet")]
    FuelAllowanceOnCooldown,
    #[msg("The spaceship has no available subsystem upgrade points to spend")]
    NoAvailableSubsystemUpgradePoint,
    #[msg("The spaceship has no available crate to pick at this moment")]
    NoCrateAvailable,
    #[msg("The switchboard function wasn't signed by the enclave")]
    FunctionValidationFailed,
    #[msg("There is a problem with the loot table")]
    InvalidLootTable,
    #[msg("The ingame spaceship wallet doesn't have enough funds to pay for the transaction")]
    InsufficientFunds,
    #[msg("The spaceship does not have sufficient unspent subsystem upgrade points to allocate")]
    InsufficientSubsystemUpgradePoints,
    #[msg("The spaceship is decked out with the maximum amount of powerups")]
    MaxPowerupScoreReached,
    #[msg("The spaceship's subsystem is at max level")]
    MaxSubsystemLevelReached,
    #[msg("The game state does not permit this action")]
    InvalidAction,
}

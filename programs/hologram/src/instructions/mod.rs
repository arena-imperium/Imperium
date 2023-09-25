// admin instructions
pub mod arena_matchmaking_settle;
pub mod create_spaceship_settle;
pub mod initialize_realm;
pub mod user_facing;

// public instructions

// bring everything in scope
pub use {
    arena_matchmaking_settle::*, create_spaceship_settle::*, initialize_realm::*, user_facing::*,
};

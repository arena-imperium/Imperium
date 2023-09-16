// admin instructions
pub mod create_spaceship;
pub mod create_spaceship_settle;
pub mod create_user_account;
pub mod initialize_realm;

// public instructions

// bring everything in scope
pub use {
    create_spaceship::*, create_spaceship_settle::*, create_user_account::*, initialize_realm::*,
};

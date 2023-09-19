pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use {anchor_lang::prelude::*, instructions::*};

#[cfg(feature = "localnet")]
declare_id!("AMXakgYy6jGM9jSmrvfywZgGcgXnMGBcxXTawY2gAT4u");
#[cfg(feature = "devnet")]
declare_id!("FsqyVQ113X1VzZhT17smtZoyAv9Jq9gJfiyMqpL59mo8");
#[cfg(feature = "mainnet-beta")]
declare_id!("Hologram1111");

pub const SHORT_LIMITED_STRING_MAX_LENGTH: usize = 64;
pub const LONG_LIMITED_STRING_MAX_LENGTH: usize = 256;

pub const RANDOMNESS_LOWER_BOUND: u32 = 1;
pub const RANDOMNESS_UPPER_BOUND: u32 = 1_000_000;
pub const SPACESHIP_RANDOMNESS_FUNCTION_FEE: u64 = 0;
pub const ARENA_MATCHMAKING_FUNCTION_FEE: u64 = 0;

// Max amount of fuel for new Spaceships
pub const BASE_MAX_FUEL: u8 = 5;
// Amount of fuel that is provided per day per spaceship
pub const DAILY_FUEL_ALLOWANCE: u8 = 3;

// Experience required for next level is equal to next_level * XP_REQUIERED_PER_LEVEL_MULT
pub const XP_REQUIERED_PER_LEVEL_MULT: u8 = 5;
// Maximum amount of xp for next level (caps the above calculation)
pub const MAX_XP_PER_LEVEL: u16 = 50;
// Maximum spaceship level -  MUST be an even number
pub const MAX_LEVEL: u8 = 16;

//  -- Statistics --
// 2 level of related stat provide 1% dodge chance
pub const DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO: u8 = 2;
// 2 level of related stat provide 1% jamming nullifying
pub const JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO: u8 = 2;

pub const BASE_DODGE_CHANCE: u8 = 5; // 5%
pub const DODGE_CHANCE_CAP: u8 = 35; // 35%

pub const BASE_JAMMING_NULLIFYING_CHANCE: u8 = 10; // 10%
pub const JAMMING_NULLIFYING_CHANCE_CAP: u8 = 75; // 75%

// Celerity is the statistic that determine who plays first during a fight (higher is better)
pub const BASE_CELERITY: u8 = 10;

pub const BASE_HULL_HITPOINTS: u16 = 75;
pub const BASE_ARMOR_HITPOINTS: u16 = 10;
pub const BASE_SHIELD_HITPOINTS: u16 = 0;
pub const HULL_HITPOINTS_PER_LEVEL: u16 = 5;
pub const ARMOR_HITPOINTS_PER_ARMOR_LAYERING_LEVEL: u16 = 20;
pub const SHIELD_HITPOINTS_PER_SHIELD_SUBSYSTEMS_LEVEL: u16 = 15;

pub const ARENA_MATCHMAKING_FUEL_COST: u8 = 1;
pub const ARENA_MATCHMAKING_LEVEL_PER_RANGE: u8 = 2;
pub const ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE: u8 = 5;

solana_security_txt::security_txt! {
    name: "Hologram",
    project_url: "https://github.com/acamill",
    contacts: "email:alexcamill@gmail.com",
    policy: "",
    preferred_languages: "en",
    auditors: "None"
}

pub const MAX_SPACESHIPS_PER_USER_ACCOUNT: usize = 25;

#[program]
pub mod hologram {
    use super::*;

    // Public IX ----------------------------------------------------------------

    /// Called to initialize a new realm.
    /// Will be called by us once at inception but we can imagine Seasonal realms afterward or player run realms.
    pub fn initialize_realm(ctx: Context<InitializeRealm>, name: String) -> Result<()> {
        instructions::initialize_realm(ctx, name)
    }

    // Create a user_account tied to a realm, this will store a player information and spaceships
    pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        instructions::create_user_account(ctx)
    }

    // Create a spaceship for a user_account. A spaceship can be though of as a Character in a RPG.
    //
    // During the instruction a request account is initialized and triggered for the spaceship_seed_generation_function.
    // The call back for this is the create_spaceship_settle IX.
    //
    // During the instruction a request account is initialized for the arena_matchmaking_function.
    pub fn create_spaceship(ctx: Context<CreateSpaceship>, name: String) -> Result<()> {
        instructions::create_spaceship(ctx, name)
    }
    // Switchboard function callback
    pub fn create_spaceship_settle(
        ctx: Context<CreateSpaceshipSettle>,
        generated_seed: u32,
    ) -> Result<()> {
        instructions::create_spaceship_settle(ctx, generated_seed)
    }
}

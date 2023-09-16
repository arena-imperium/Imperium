pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use {anchor_lang::prelude::*, instructions::*};

#[cfg(feature = "localnet")]
declare_id!("AMXakgYy6jGM9jSmrvfywZgGcgXnMGBcxXTawY2gAT4u");
#[cfg(feature = "devnet")]
declare_id!("5EDtp1G7GkGEGWCKJf6np2gC7kdCdJ7Xi59fSmqEEPfX");
#[cfg(feature = "mainnet-beta")]
declare_id!("Hologram1111");

pub const RANDOMNESS_LOWER_BOUND: u32 = 1;
pub const RANDOMNESS_UPPER_BOUND: u32 = 1_000_000;
pub const RANDOMNESS_LAMPORT_COST: u64 = 0;

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
    pub fn initialize_realm(ctx: Context<InitializeRealm>, name: String) -> Result<()> {
        instructions::initialize_realm(ctx, name)
    }

    pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        instructions::create_user_account(ctx)
    }

    pub fn create_spaceship(ctx: Context<CreateSpaceship>, name: String) -> Result<()> {
        instructions::create_spaceship(ctx, name)
    }

    pub fn create_spaceship_settle(
        ctx: Context<CreateSpaceshipSettle>,
        generated_seed: u32,
    ) -> Result<()> {
        instructions::create_spaceship_settle(ctx, generated_seed)
    }

    // Views
}

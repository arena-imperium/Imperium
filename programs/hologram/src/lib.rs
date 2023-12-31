pub mod engine;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use {anchor_lang::prelude::*, instructions::*};

#[cfg(feature = "localnet")]
declare_id!("GiN7xhFgwGTciboPZHyGu2v16LDezaXgkhMW9Pv5xiet");
#[cfg(feature = "devnet")]
declare_id!("BDWigaQZHnLxc2vZ5MqvorW5ki5uFpaNWezeyUoBaZgj");
#[cfg(feature = "mainnet-beta")]
declare_id!("Hologram1111");

pub const SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION: u8 = 150; // minimum value ~1mn
pub const STARTING_IMPERIAL_CREDITS: u8 = NI_PRICE + 5;
pub const MAX_ORDNANCE: u8 = 16;
// PowerUp score is the sum of all the powerups for a ship.
pub const CURRENCY_REWARD_FOR_ARENA_WINNER: u8 = 3;
pub const CURRENCY_REWARD_FOR_ARENA_LOOSER: u8 = 1;
pub const RANDOMNESS_LOWER_BOUND: u32 = 1;
pub const RANDOMNESS_UPPER_BOUND: u32 = 1_000_000;
// Max amount of fuel for new Spaceships
pub const BASE_MAX_FUEL: u8 = 5;
// Amount of fuel that is provided per period per spaceship
pub const FUEL_ALLOWANCE_AMOUNT: u8 = 3;
// How often the fuel allowance is provided
pub const FUEL_ALLOWANCE_COOLDOWN: i64 = 24 * 60 * 60; // 24 hours in seconds
pub const BASE_DODGE_CHANCE: u8 = 5; // 5%
pub const DODGE_CHANCE_CAP: u8 = 35; // 35%
pub const BASE_JAMMING_NULLIFYING_CHANCE: u8 = 10; // 10%
pub const JAMMING_NULLIFYING_CHANCE_CAP: u8 = 75; // 75%
pub const BASE_HULL_HITPOINTS: u8 = 30;
pub const BASE_SHIELD_LAYERS: u8 = 0;
pub const ARENA_MATCHMAKING_FUEL_COST: u8 = 1;
pub const ARENA_MATCHMAKING_ORDNANCE_PER_RANGE: u8 = 2;
pub const ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE: u8 = 5;
pub const MATCH_MAX_TURN: u16 = 1000;
pub const CHARGE_PER_TURN: u8 = 1;
pub const HEAT_DISSIPATION_PER_TURN: u8 = 1;
pub const BASE_HEAT: u8 = 5;
pub const BASE_JAM_CHANCE: u8 = 100;

solana_security_txt::security_txt! {
    name: "Hologram",
    project_url: "https://github.com/acamill",
    contacts: "email:alexcamill@gmail.com",
    policy: "",
    preferred_languages: "en",
    auditors: "None"
}

pub const MAX_SPACESHIPS_PER_USER_ACCOUNT: usize = 10;

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

    // Queue for matchmaking in the arena (softcore)
    #[allow(unused_variables)]
    pub fn arena_matchmaking(
        ctx: Context<ArenaMatchmaking>,
        spaceship_index: u8,
        faction: Faction,
    ) -> Result<()> {
        instructions::arena_matchmaking(ctx, faction)
    }
    // Switchboard function callback
    // pairs up the spaceship with another one from the matchmaking queue and start the fight
    pub fn arena_matchmaking_settle(
        ctx: Context<ArenaMatchmakingSettle>,
        generated_seed: u32,
        faction: Faction,
    ) -> Result<()> {
        instructions::arena_matchmaking_settle(ctx, generated_seed, faction)
    }

    // Once per FUEL_ALLOWANCE_COOLDOWN players can claim free Fuel for each of their spaceships
    #[allow(unused_variables)]
    pub fn claim_fuel_allowance(
        ctx: Context<ClaimFuelAllowance>,
        spaceship_index: u8,
    ) -> Result<()> {
        instructions::claim_fuel_allowance(ctx)
    }

    // Purchase a crate, will roll for a RNG based drop to power up the spaceship
    #[allow(unused_variables)]
    pub fn pick_crate(
        ctx: Context<PickCrate>,
        spaceship_index: u8,
        crate_type: CrateType,
    ) -> Result<()> {
        instructions::pick_crate(ctx, crate_type)
    }
    // Switchboard function callback
    // given the choosen crate, pick the reward using the generated_seed
    pub fn pick_crate_settle(
        ctx: Context<PickCrateSettle>,
        generated_seed: u32,
        crate_type: CrateType,
    ) -> Result<()> {
        instructions::pick_crate_settle(ctx, generated_seed, crate_type)
    }
}

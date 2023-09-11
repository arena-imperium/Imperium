pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use {anchor_lang::prelude::*, instructions::*};

declare_id!("AMXakgYy6jGM9jSmrvfywZgGcgXnMGBcxXTawY2gAT4u");

solana_security_txt::security_txt! {
    name: "Hologram",
    project_url: "https://github.com/acamill",
    contacts: "email:alexcamill@gmail.com",
    policy: "",
    preferred_languages: "en",
    auditors: "None"
}

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

}

use getrandom::register_custom_getrandom;
fn custom_getrandom(buf: &mut [u8]) -> std::result::Result<(), getrandom::Error> {
    // Improve probably. Maybe use real random later
    buf.copy_from_slice(Clock::get().unwrap().slot.to_le_bytes().as_ref());
    return Ok(());
}

register_custom_getrandom!(custom_getrandom);

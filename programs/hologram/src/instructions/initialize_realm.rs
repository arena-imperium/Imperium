use {
    crate::{
        error::HologramError,
        state::{MatchmakingQueue, Realm},
        utils::LimitedString,
        ARENA_MATCHMAKING_LEVEL_PER_RANGE, ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE, MAX_LEVEL,
    },
    anchor_lang::prelude::*,
    switchboard_solana::FunctionAccountData,
};
// The realm represent the game world. It is the top level of the game hierarchy.

#[derive(Accounts)]
#[instruction(name:String)]
pub struct InitializeRealm<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: any
    pub admin: AccountInfo<'info>,

    #[account(
        init,
        payer=payer,
        seeds=[b"realm", name.as_bytes()],
        bump,
        space = Realm::LEN + (MAX_LEVEL as usize / ARENA_MATCHMAKING_LEVEL_PER_RANGE as usize * std::mem::size_of::<MatchmakingQueue>()),
    )]
    pub realm: Account<'info, Realm>,

    // Randomness generator function from Switchboard (custom)
    #[account(
        constraint =
            // Ensure our authority owns this function
            switchboard_function.load()?.authority == *admin.key &&
            // Ensure custom requests are allowed
            !switchboard_function.load()?.requests_disabled
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_realm(ctx: Context<InitializeRealm>, name: String) -> Result<()> {
    // Checks
    {
        // verify input parameters
        require!(
            name.len() <= LimitedString::MAX_LENGTH,
            HologramError::LimitedStringLengthExceeded
        );
    }

    // Initialize Realm account
    {
        ctx.accounts.realm.bump = *ctx.bumps.get("realm").ok_or(ProgramError::InvalidSeeds)?;
        ctx.accounts.realm.name = LimitedString::new(name);
        ctx.accounts.realm.admin = ctx.accounts.admin.key();
        ctx.accounts
            .realm
            .switchboard_info
            .spaceship_seed_generation_function = ctx.accounts.switchboard_function.key();
        ctx.accounts.realm.switchboard_info.authority = ctx.accounts.admin.key();
    }

    // Initialize arena matchmaking queue
    {
        let realm = &mut ctx.accounts.realm;
        for i in (0..MAX_LEVEL).step_by(ARENA_MATCHMAKING_LEVEL_PER_RANGE as usize) {
            realm.arena_matchmaking_queue.push(MatchmakingQueue {
                up_to_level: i + ARENA_MATCHMAKING_LEVEL_PER_RANGE,
                spaceships: [None; ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE as usize],
                matchmaking_request_count: 0,
            });
        }
    }

    Ok(())
}

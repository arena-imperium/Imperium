use {
    crate::{
        error::HologramError,
        state::{MatchmakingQueue, Realm},
        utils::LimitedString,
        ARENA_MATCHMAKING_ORDNANCE_PER_RANGE, ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE, MAX_ORDNANCE,
    },
    anchor_lang::prelude::*,
    switchboard_solana::FunctionAccountData,
};

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
        space = Realm::LEN + (MAX_ORDNANCE as usize / ARENA_MATCHMAKING_ORDNANCE_PER_RANGE as usize * std::mem::size_of::<MatchmakingQueue>()),
    )]
    pub realm: Account<'info, Realm>,

    // spaceship seed generation function
    #[account(
        constraint =
            // Ensure our authority owns this function
            // spaceship_seed_generation_function.load()?.authority == *admin.key &&
            // Ensure custom requests are allowed
            !spaceship_seed_generation_function.load()?.requests_disabled
    )]
    pub spaceship_seed_generation_function: AccountLoader<'info, FunctionAccountData>,

    /// arena matchmaking function
    #[account(
        constraint =
            // Ensure our authority owns this function
            // arena_matchmaking_function.load()?.authority == *admin.key &&
            // Ensure custom requests are allowed
            !arena_matchmaking_function.load()?.requests_disabled
    )]
    pub arena_matchmaking_function: AccountLoader<'info, FunctionAccountData>,

    /// crate picking function
    #[account(
        constraint =
            // Ensure our authority owns this function
            // arena_matchmaking_function.load()?.authority == *admin.key &&
            // Ensure custom requests are allowed
            !crate_picking_function.load()?.requests_disabled
    )]
    pub crate_picking_function: AccountLoader<'info, FunctionAccountData>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct RealmInitialized {
    pub name: String,
    pub pda: Pubkey,
    pub admin: Pubkey,
    pub spaceship_seed_generation_function: Pubkey,
    pub arena_matchmaking_function: Pubkey,
    pub crate_picking_function: Pubkey,
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
            .spaceship_seed_generation_function =
            ctx.accounts.spaceship_seed_generation_function.key();
        ctx.accounts
            .realm
            .switchboard_info
            .arena_matchmaking_function = ctx.accounts.arena_matchmaking_function.key();
        ctx.accounts.realm.switchboard_info.crate_picking_function =
            ctx.accounts.crate_picking_function.key();
        ctx.accounts.realm.switchboard_info.authority = ctx.accounts.admin.key();
    }

    // Initialize arena matchmaking queue
    {
        let realm = &mut ctx.accounts.realm;
        // Note: spaceships starts at 2 ordnance and there will be a queue for 0-2(exclusive) ordnance but that
        // way all case are covered (a player plucking modules even the civilian ones is still supported)
        for i in (0..MAX_ORDNANCE).step_by(ARENA_MATCHMAKING_ORDNANCE_PER_RANGE as usize) {
            realm.arena_matchmaking_queue.push(MatchmakingQueue {
                up_to_ordnance: i + ARENA_MATCHMAKING_ORDNANCE_PER_RANGE,
                spaceships: [None; ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE as usize],
                matchmaking_request_count: 0,
            });
        }
    }

    emit!(RealmInitialized {
        name: ctx.accounts.realm.name.to_string(),
        pda: ctx.accounts.realm.key(),
        admin: ctx.accounts.admin.key(),
        spaceship_seed_generation_function: ctx.accounts.spaceship_seed_generation_function.key(),
        arena_matchmaking_function: ctx.accounts.arena_matchmaking_function.key(),
        crate_picking_function: ctx.accounts.crate_picking_function.key(),
    });

    Ok(())
}

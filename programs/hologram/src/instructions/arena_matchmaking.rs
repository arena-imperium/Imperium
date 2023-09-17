use crate::ARENA_MATCHMAKING_FUNCTION_FEE;

use {
    crate::{
        error::HologramError,
        state::{spaceship, Realm, SpaceShip, SpaceShipLite, UserAccount},
        utils::RandomNumberGenerator,
    },
    anchor_lang::prelude::*,
    spaceship::Hull,
    switchboard_solana::prelude::*,
};


#[derive(Accounts)]
pub struct ArenaMatchmaking<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
        has_one = admin,
    )]
    pub realm: Box<Account<'info, Realm>>,

    /// CHECK: by the realm account. Used with switchboard function
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        realloc = UserAccount::LEN + std::mem::size_of::<SpaceShipLite>() * user_account.spaceships.len(),
        realloc::payer = user,
        realloc::zero = false,
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Box<Account<'info, UserAccount>>,

    #[account(
        init,
        payer=user,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        bump,
        space = SpaceShip::LEN
    )]
    pub spaceship: Account<'info, SpaceShip>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,

    /// CHECK: validated by Switchboard CPI
    #[account(
        mut, 
        // validate that we use the realm custom switchboard function for the arena matchmaking
        constraint = realm.switchboard_info.arena_matchmaking_function == switchboard_function.key() && !switchboard_function.load()?.requests_disabled
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,

    // The Switchboard Function Request account we will create with a CPI.
    // Should be an empty keypair with no lamports or data.
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_request.data_len() == 0 && switchboard_request.lamports() == 0
      )]
    pub switchboard_request: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_request_escrow.data_len() == 0 && switchboard_request_escrow.lamports() == 0
      )]
    pub switchboard_request_escrow: AccountInfo<'info>,

    // User WSOL token account to pay for the function execution
    #[account(
      init_if_needed,
      payer = user,
      associated_token::mint = switchboard_mint,
      associated_token::authority = user,
    )]
    pub user_wsol_token_account: Account<'info, TokenAccount>,

    // WSOL Mint, and function related accounts used to pay for the switchboard function execution
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub switchboard_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: SWITCHBOARD_ATTESTATION_PROGRAM
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn arena_matchmaking(ctx: Context<ArenaMatchmaking>) -> Result<()> {
    // Validations
    {
        // verify that the user has not registered for the arena yet
        require!(
            ctx.accounts.spaceship.arena_matchmaking.status != crate::state::SwitchboardFunctionRequestStatus::Requested,
            HologramError::ArenaMatchmakingAlreadyRequested
        );

    }

        // wrap GUESS_COST lamports on user wallet, if needed, to prepare for the function execution cost
    {
        // Only proceed if the user doesn't have enough lamports to pay for the function execution
        if ctx.accounts.user_wsol_token_account.amount < ARENA_MATCHMAKING_FUNCTION_FEE {
            switchboard_solana::wrap_native(
                &ctx.accounts.system_program,
                &ctx.accounts.token_program,
                &ctx.accounts.user_wsol_token_account,
                &ctx.accounts.user,
                &[&[
                    b"realm",
                    ctx.accounts.realm.name.to_bytes(),
                    &[ctx.accounts.realm.bump],
                ]],
                ARENA_MATCHMAKING_FUNCTION_FEE
                    .checked_sub(ctx.accounts.user_wsol_token_account.amount)
                    .unwrap(),
            )?;
            // Reload the user wallet account to get the new amount
            ctx.accounts.user_wsol_token_account.reload()?;
        }
    }

    // pay 1 fuel to entry
    {
        
    }
    Ok(())
}

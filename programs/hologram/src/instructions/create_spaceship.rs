use {
    crate::{
        error::HologramError,
        state::{Realm, SpaceShip, SpaceShipLite, UserAccount},
        utils::LimitedString,
        MAX_SPACESHIPS_PER_USER_ACCOUNT, RANDOMNESS_LAMPORT_COST,
    },
    switchboard_solana::prelude::*,
};

// @TODO: Handle fail switchboard request eventually
// @TODO: Create a transfer/close spaceship IX (remember to handle the switchboard_request account, holds rent)

#[derive(Accounts)]
pub struct CreateSpaceship<'info> {
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
        // validate that we use the realm custom switchboard function
        constraint = realm.switchboard_info.spaceship_seed_generation_function == switchboard_function.key() && !switchboard_function.load()?.requests_disabled
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

pub fn create_spaceship(ctx: Context<CreateSpaceship>, name: String) -> Result<()> {
    // Validations
    {
        // has not reached max spaceships per user_account
        require!(
            ctx.accounts.user_account.spaceships.len() < MAX_SPACESHIPS_PER_USER_ACCOUNT,
            HologramError::SpaceshipsLimitExceeded
        );
        // verify input parameters
        require!(
            name.len() <= LimitedString::MAX_LENGTH,
            HologramError::LimitedStringLengthExceeded
        );
        // verify than there is no other spaceship with the same name for that user_account
        require!(
            !ctx.accounts
                .user_account
                .spaceships
                .iter()
                .any(|s| s.name.to_string() == name),
            HologramError::SpaceshipNameAlreadyExists
        );
        // // verify that there is no pending request already
        // require!(
        //     ctx.accounts.spaceship.randomness.status == crate::state::RandomnessStatus::None,
        //     HologramError::SpaceshipRandomnessAlreadyRequested
        // );
    }

    // @TODO: add a fee to create a spaceship

    // Initialize the new SpaceShip account
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.bump = *ctx
            .bumps
            .get("spaceship")
            .ok_or(ProgramError::InvalidSeeds)?;
        spaceship.owner = *ctx.accounts.user.key;
        spaceship.name = LimitedString::new(name);
    }

    // wrap GUESS_COST lamports on user wallet, if needed, to prepare for the function execution cost
    {
        // Only proceed if the user doesn't have enough lamports to pay for the function execution
        if ctx.accounts.user_wsol_token_account.amount < RANDOMNESS_LAMPORT_COST {
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
                RANDOMNESS_LAMPORT_COST
                    .checked_sub(ctx.accounts.user_wsol_token_account.amount)
                    .unwrap(),
            )?;
            // Reload the user wallet account to get the new amount
            ctx.accounts.user_wsol_token_account.reload()?;
        }
    }

    // Init and Trigger the Switchboard request
    // This will instruct the off-chain oracles to execute the docker container and relay
    // the result back to our program via the 'create_spaceship_settle' instruction.
    {
        let request_params = format!(
            "PID={},LOWER_BOUND={},UPPER_BOUND={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={}",
            crate::id(),
            crate::RANDOMNESS_LOWER_BOUND,
            crate::RANDOMNESS_UPPER_BOUND,
            ctx.accounts.user.key(),
            ctx.accounts.realm.key(),
            ctx.accounts.user_account.key(),
            ctx.accounts.spaceship.key()
        );

        let request_init_ctx = FunctionRequestInitAndTrigger {
            request: ctx.accounts.switchboard_request.clone(),
            function: ctx.accounts.switchboard_function.to_account_info(),
            escrow: ctx.accounts.switchboard_request_escrow.to_account_info(),
            mint: ctx.accounts.switchboard_mint.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        request_init_ctx.invoke(
            ctx.accounts.switchboard_program.clone(),
            // bounty - optional fee to reward oracles for priority processing
            // default: 0 lamports
            None,
            // slots_until_expiration - optional max number of slots the request can be processed in
            // default: 2250 slots, ~ 15 min at 400 ms/slot
            // minimum: 150 slots, ~ 1 min at 400 ms/slot
            None,
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(512),
            // container_params - the container params
            // default: empty vec
            Some(request_params.into_bytes()),
            // garbage_collection_slot - the slot when the request can be closed by anyone and is considered dead
            // default: None, only authority can close the request
            None,
            // valid_after_slot - schedule a request to execute in N slots
            // default: 0 slots, valid immediately for oracles to process
            None,
        )?;

        // update the spaceship randomness state
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.randomness.switchboard_request = ctx.accounts.switchboard_request.key();
        spaceship.randomness.status = crate::state::RandomnessStatus::Pending;
        spaceship.randomness.original_seed = 0;
        spaceship.randomness.current_seed = 0;
        spaceship.randomness.iteration = 0;
    }

    Ok(())
}

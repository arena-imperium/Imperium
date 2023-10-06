use {
    crate::{
        error::HologramError,
        state::{
            Module, Realm, SpaceShip, SpaceShipLite, SwitchboardFunctionRequestStatus, UserAccount,
        },
        utils::LimitedString,
        BASE_MAX_FUEL, MAX_SPACESHIPS_PER_USER_ACCOUNT,
    },
    anchor_lang::prelude::*,
    anchor_spl::associated_token::AssociatedToken,
    switchboard_solana::{
        self, AttestationProgramState, AttestationQueueAccountData, FunctionAccountData, Token,
        SWITCHBOARD_ATTESTATION_PROGRAM_ID,
    },
};
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

    /// CHECK: validated by the realm admin
    #[account(constraint = admin.key() == realm.admin)]
    pub admin: AccountInfo<'info>,

    // Note: resize is made in the request rather than settle for simplicity
    // this cannot backfire as the len is increased in the settle part.
    #[account(
        mut,
        realloc = UserAccount::LEN + std::mem::size_of::<SpaceShipLite>() * (user_account.spaceships.len() + 1),
        realloc::payer = user,
        realloc::zero = false,
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Box<Account<'info, UserAccount>>,

    #[account(
        init_if_needed, // this might be called again if the settlement ix fails
        payer=user,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        bump,
        space = SpaceShip::LEN + std::mem::size_of::<Module>(), // make space for the starter civilian weapon module
    )]
    pub spaceship: Box<Account<'info, SpaceShip>>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,

    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        // validate that we use the realm custom switchboard function for spaceship seed generation
        constraint = realm.switchboard_info.spaceship_seed_generation_function == spaceship_seed_generation_function.key() && !spaceship_seed_generation_function.load()?.requests_disabled
    )]
    pub spaceship_seed_generation_function: AccountLoader<'info, FunctionAccountData>,

    // The Switchboard Function Request account we will create with a CPI.
    // Should be an empty keypair with no lamports or data.
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_ssgf_request.data_len() == 0 && switchboard_ssgf_request.lamports() == 0
      )]
    pub switchboard_ssgf_request: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_ssgf_request_escrow.data_len() == 0 && switchboard_ssgf_request_escrow.lamports() == 0
      )]
    pub switchboard_ssgf_request_escrow: AccountInfo<'info>,

    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        // validate that we use the realm custom switchboard function for arena match making
        constraint = realm.switchboard_info.arena_matchmaking_function == arena_matchmaking_function.key() && !arena_matchmaking_function.load()?.requests_disabled
    )]
    pub arena_matchmaking_function: AccountLoader<'info, FunctionAccountData>,

    // The Switchboard Function Request account we will create with a CPI.
    // Should be an empty keypair with no lamports or data.
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_amf_request.data_len() == 0 && switchboard_amf_request.lamports() == 0
      )]
    pub switchboard_amf_request: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_amf_request_escrow.data_len() == 0 && switchboard_amf_request_escrow.lamports() == 0
      )]
    pub switchboard_amf_request_escrow: AccountInfo<'info>,

    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        // validate that we use the realm custom switchboard function for arena match making
        constraint = realm.switchboard_info.arena_matchmaking_function == arena_matchmaking_function.key() && !arena_matchmaking_function.load()?.requests_disabled
    )]
    pub crate_picking_function: AccountLoader<'info, FunctionAccountData>,

    // The Switchboard Function Request account we will create with a CPI.
    // Should be an empty keypair with no lamports or data.
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_amf_request.data_len() == 0 && switchboard_amf_request.lamports() == 0
      )]
    pub switchboard_cpf_request: AccountInfo<'info>,

    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_amf_request_escrow.data_len() == 0 && switchboard_amf_request_escrow.lamports() == 0
      )]
    pub switchboard_cpf_request_escrow: AccountInfo<'info>,

    // WSOL Mint, and function related accounts used to pay for the switchboard function execution
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub switchboard_mint: Box<Account<'info, switchboard_solana::Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    /// CHECK: SWITCHBOARD_ATTESTATION_PROGRAM
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard_program: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn create_spaceship(ctx: Context<CreateSpaceship>, name: String) -> Result<()> {
    // cancel pending switchboard function request if stale
    {
        let spaceship = &mut ctx.accounts.spaceship;
        let current_slot = Realm::get_slot()?;
        if spaceship
            .randomness
            .switchboard_request_info
            .request_is_expired(current_slot)
        {
            spaceship.randomness.switchboard_request_info.status =
                SwitchboardFunctionRequestStatus::Expired { slot: current_slot };
        }
    }

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
        // verify that there is no pending request already
        require!(
            matches!(
                ctx.accounts
                    .spaceship
                    .randomness
                    .switchboard_request_info
                    .status,
                crate::state::SwitchboardFunctionRequestStatus::None
            ),
            HologramError::SpaceshipRandomnessAlreadyRequested
        );
    }

    // @TODO: add a fee to create a spaceship if rolling premium hulls

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

    // init the request account for the arena_matchmaking_function. Not used in this context, but
    // will be ready for future calls to arena_matchmaking IX.
    #[cfg(not(any(test, feature = "testing")))]
    {
        use switchboard_solana::FunctionRequestInit;

        // Create the Switchboard request account.
        let request_init_ctx = FunctionRequestInit {
            request: ctx.accounts.switchboard_amf_request.clone(),
            authority: ctx.accounts.user_account.to_account_info(),
            function: ctx.accounts.arena_matchmaking_function.to_account_info(),
            function_authority: None, // only needed if switchboard_function.requests_require_authorization is enabled
            escrow: ctx
                .accounts
                .switchboard_amf_request_escrow
                .to_account_info(),
            mint: ctx.accounts.switchboard_mint.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        let request_params = format!(
            "PID={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={},OS_1_PDA={},OS_2_PDA={},OS_3_PDA={},OS_4_PDA={},OS_5_PDA={}",
            crate::id(),
            ctx.accounts.user.key(),
            ctx.accounts.realm.key(),
            ctx.accounts.user_account.key(),
            ctx.accounts.spaceship.key(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        );
        request_init_ctx.invoke(
            ctx.accounts.switchboard_program.clone(),
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(400),
            // container_params - the container params
            // default: empty vec
            Some(request_params.into_bytes()),
            // garbage_collection_slot - the slot when the request can be closed by anyone and is considered dead
            // default: None, only authority can close the request
            None,
        )?;
    }

    // update the spaceship arena_matchmaking state
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.arena_matchmaking.switchboard_request_info.account =
            ctx.accounts.switchboard_amf_request.key();
        spaceship.arena_matchmaking.switchboard_request_info.status =
            SwitchboardFunctionRequestStatus::None;
    }

    // init the request account for the crate_picking_function. Not used in this context, but
    // will be ready for future calls to pick_crate IX.
    #[cfg(not(any(test, feature = "testing")))]
    {
        use {crate::CrateType, switchboard_solana::FunctionRequestInit};

        // Create the Switchboard request account.
        let request_init_ctx = FunctionRequestInit {
            request: ctx.accounts.switchboard_cpf_request.clone(),
            authority: ctx.accounts.user_account.to_account_info(),
            function: ctx.accounts.crate_picking_function.to_account_info(),
            function_authority: None, // only needed if switchboard_function.requests_require_authorization is enabled
            escrow: ctx
                .accounts
                .switchboard_cpf_request_escrow
                .to_account_info(),
            mint: ctx.accounts.switchboard_mint.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        let request_params = format!(
            "PID={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={},CRATE_TYPE{}",
            crate::id(),
            ctx.accounts.user.key(),
            ctx.accounts.realm.key(),
            ctx.accounts.user_account.key(),
            ctx.accounts.spaceship.key(),
            CrateType::NavyIssue as u8,
        );
        request_init_ctx.invoke(
            ctx.accounts.switchboard_program.clone(),
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(180),
            // container_params - the container params
            // default: empty vec
            Some(request_params.into_bytes()),
            // garbage_collection_slot - the slot when the request can be closed by anyone and is considered dead
            // default: None, only authority can close the request
            None,
        )?;
    }

    // update the spaceship crate_picking state
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.crate_picking.switchboard_request_info.account =
            ctx.accounts.switchboard_cpf_request.key();
        spaceship.crate_picking.switchboard_request_info.status =
            SwitchboardFunctionRequestStatus::None;
    }

    // Init and Trigger the request account for the spaceship_seed_generation_function
    // This will instruct the off-chain oracles to execute the docker container and relay
    // the result back to our program via the 'create_spaceship_settle' instruction.
    #[cfg(not(any(test, feature = "testing")))]
    {
        use {
            crate::SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION,
            switchboard_solana::FunctionRequestInitAndTrigger,
        };

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

        let request_init_and_trigger_ctx = FunctionRequestInitAndTrigger {
            request: ctx.accounts.switchboard_ssgf_request.clone(),
            authority: ctx.accounts.admin.to_account_info(),
            function: ctx
                .accounts
                .spaceship_seed_generation_function
                .to_account_info(),
            function_authority: None, // only needed if switchboard_function.requests_require_authorization is enabled
            escrow: ctx
                .accounts
                .switchboard_ssgf_request_escrow
                .to_account_info(),
            mint: ctx.accounts.switchboard_mint.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        request_init_and_trigger_ctx.invoke(
            ctx.accounts.switchboard_program.clone(),
            // bounty - optional fee to reward oracles for priority processing
            // default: 0 lamports
            None,
            // slots_until_expiration - optional max number of slots the request can be processed in
            // default: 2250 slots, ~ 15 min at 400 ms/slot
            // minimum: 150 slots, ~ 1 min at 400 ms/slot
            Some(SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION as u64),
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(160),
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
    }

    // update the spaceship randomness state
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.randomness.switchboard_request_info.account =
            ctx.accounts.switchboard_ssgf_request.key();
        spaceship.randomness.switchboard_request_info.status =
            SwitchboardFunctionRequestStatus::Requested {
                slot: Realm::get_slot()?,
            };
        // randomness fields defaulted to 0 for now, soon updated in the settle callback
    }

    // initialize remaining spaceship fields
    {
        let spaceship = &mut ctx.accounts.spaceship;

        spaceship.fuel.max = BASE_MAX_FUEL;
        spaceship.fuel.current = BASE_MAX_FUEL;
        spaceship.fuel.daily_allowance_last_collection = Realm::get_time()?;

        // hull is rolled during settle callback
    }

    Ok(())
}

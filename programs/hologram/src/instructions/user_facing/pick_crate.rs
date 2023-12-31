use {
    crate::{
        error::HologramError,
        instructions::{CrateType, BMC_PRICE, NI_PRICE, PC_PRICE},
        state::{
            Drone, Module, Mutation, Realm, SpaceShip, SpaceShipLite,
            SwitchboardFunctionRequestStatus, UserAccount,
        },
        MAX_ORDNANCE,
    },
    anchor_lang::prelude::*,
    switchboard_solana::{
        AttestationProgramState, AttestationQueueAccountData, FunctionAccountData,
        SWITCHBOARD_ATTESTATION_PROGRAM_ID,
    },
};

#[derive(Accounts)]
#[instruction(spaceship_index:u8)]
pub struct PickCrate<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Box<Account<'info, Realm>>,

    #[account(
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Box<Account<'info, UserAccount>>,

    // Note: Pre-emptively resize the modules/drones/mutations arrays to avoid reallocating them in the settle instruction
    // It complicates things to do so in the settle due to the payer required for reallocating
    #[account(
        mut,
        realloc = SpaceShip::LEN + std::mem::size_of::<Module>() * (spaceship.modules.len() + 1) + std::mem::size_of::<Drone>() * (spaceship.drones.len() + 1) + std::mem::size_of::<Mutation>() * (spaceship.mutations.len() + 1),
        realloc::payer = user,
        realloc::zero = false,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), spaceship_index.to_le_bytes().as_ref()],
        bump = spaceship.bump,
    )]
    pub spaceship: Box<Account<'info, SpaceShip>>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,

    /// CHECK: validated by Switchboard CPI
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,

    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        // validate that we use the realm custom switchboard function for the arena matchmaking
        constraint = realm.switchboard_info.crate_picking_function == crate_picking_function.key() && !crate_picking_function.load()?.requests_disabled
    )]
    pub crate_picking_function: AccountLoader<'info, FunctionAccountData>,

    /// CHECK: validated by Switchboard CPI
    #[account(mut)]
    pub switchboard_request: AccountInfo<'info>,

    /// CHECK:validated by Switchboard CPI
    #[account(mut)]
    pub switchboard_request_escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    /// CHECK: SWITCHBOARD_ATTESTATION_PROGRAM
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard_program: AccountInfo<'info>,
}

#[event]
pub struct PickCrateRequested {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
    pub crate_type: CrateType,
}

#[event]
pub struct PickCrateSuccess {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
    pub crate_type: CrateType,
    pub seed: u32,
}

#[event]
pub struct PickCrateFailed {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

#[allow(unused_variables)] // due to #cfg[]
pub fn pick_crate(ctx: Context<PickCrate>, crate_type: CrateType) -> Result<()> {
    // cancel pending switchboard function request if stale
    {
        let spaceship = &mut ctx.accounts.spaceship;
        let current_slot = Realm::get_slot()?;
        if spaceship
            .crate_picking
            .switchboard_request_info
            .request_is_expired(current_slot)
        {
            spaceship.crate_picking.switchboard_request_info.status =
                SwitchboardFunctionRequestStatus::Expired { slot: current_slot };
        }
        emit!(PickCrateFailed {
            realm_name: ctx.accounts.realm.name.to_string().clone(),
            user: *ctx.accounts.user.key,
            spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship),
        });
    }

    // Validations
    {
        // verify that the user Ordnance is not maxxed
        require!(
            ctx.accounts.spaceship.ordnance() < MAX_ORDNANCE,
            HologramError::MaxOrdnanceReached
        );

        // verify that the spaceship.wallet contains the necessary amount of currency matching the price
        // this is done in the settlement too
        let crate_price = match crate_type {
            CrateType::NavyIssue => NI_PRICE,
            CrateType::PirateContraband => PC_PRICE,
            CrateType::BiomechanicalCache => BMC_PRICE,
        };
        let available_balance = ctx
            .accounts
            .spaceship
            .wallet
            .get_balance(crate_type.payment_currency());
        require!(
            available_balance >= crate_price as u16,
            HologramError::InsufficientFunds
        );

        // verify that the user is not in the process of requesting to pick a crate already
        require!(
            !ctx.accounts
                .spaceship
                .crate_picking
                .switchboard_request_info
                .is_requested(),
            HologramError::CratePickingAlreadyRequested
        );
    }

    #[cfg(not(any(test, feature = "testing")))]
    {
        use {
            crate::SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION,
            switchboard_solana::{FunctionRequestSetConfig, FunctionRequestTrigger},
        };

        let realm_key = ctx.accounts.realm.key();
        let user_account_seed = &[
            b"user_account",
            realm_key.as_ref(),
            ctx.accounts.user.key.as_ref(),
            &[ctx.accounts.user_account.bump],
        ];
        // Update the switchboard function parameters
        {
            let request_set_config_ctx = FunctionRequestSetConfig {
                request: ctx.accounts.switchboard_request.clone(),
                authority: ctx.accounts.user_account.to_account_info(),
            };
            let request_params = format!(
                "PID={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={},CRATE_TYPE{}",
                crate::id(),
                ctx.accounts.user.key(),
                realm_key,
                ctx.accounts.user_account.key(),
                ctx.accounts.spaceship.key(),
                crate_type as u8,
            );

            request_set_config_ctx.invoke_signed(
                ctx.accounts.switchboard_program.clone(),
                request_params.into_bytes(),
                false,
                &[user_account_seed],
            )?;
            msg!("Switchboard function parameters updated");
        }

        // Trigger the request account for the crate_picking_function
        // This will instruct the off-chain oracles to execute the docker container and relay
        // the result back to our program via the 'arena_matchmaking_settle' instruction.
        {
            let request_trigger_ctx = FunctionRequestTrigger {
                request: ctx.accounts.switchboard_request.clone(),
                authority: ctx.accounts.user_account.to_account_info(),
                escrow: ctx.accounts.switchboard_request_escrow.to_account_info(),
                function: ctx.accounts.crate_picking_function.to_account_info(),
                state: ctx.accounts.switchboard_state.to_account_info(),
                attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
                payer: ctx.accounts.user.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };

            request_trigger_ctx.invoke_signed(
                ctx.accounts.switchboard_program.clone(),
                // bounty - optional fee to reward oracles for priority processing
                // default: 0 lamports
                None,
                // slots_until_expiration - optional max number of slots the request can be processed in
                // default: 2250 slots, ~ 15 min at 400 ms/slot
                // minimum: 150 slots, ~ 1 min at 400 ms/slot
                Some(SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION as u64),
                // valid_after_slot - schedule a request to execute in N slots
                // default: 0 slots, valid immediately for oracles to process
                None,
                &[user_account_seed],
            )?;
            msg!("Switchboard function request triggered");
        }
    }

    // update spaceship crate_picking status
    {
        ctx.accounts
            .spaceship
            .crate_picking
            .switchboard_request_info
            .status = SwitchboardFunctionRequestStatus::Requested {
            slot: Realm::get_slot()?,
        };
    }

    emit!(PickCrateRequested {
        realm_name: ctx.accounts.realm.name.to_string().clone(),
        user: *ctx.accounts.user.key,
        spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship),
        crate_type,
    });
    Ok(())
}

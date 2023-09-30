use crate::state::{Module, Drone, Mutation};

use {
    crate::{
        instructions::CrateType,
        state::{
            Realm, SpaceShip, UserAccount,
        },
        error::HologramError,
        state::spaceship::SwitchboardFunctionRequestStatus
    },
    anchor_lang::prelude::*,
    switchboard_solana::{AttestationQueueAccountData, AttestationProgramState, FunctionAccountData, SWITCHBOARD_ATTESTATION_PROGRAM_ID},
};

#[derive(Accounts)]
pub struct PickCrate<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: validated by the realm admin
    #[account(constraint = admin.key() == realm.admin)]
    pub admin: AccountInfo<'info>,

    #[account(
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Box<Account<'info, Realm>>,

    #[account(
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    // Note: Pre-emptively resize the modules/drones/mutations arrays to avoid reallocating them in the settle instruction
    // It complicates things to do so in the settle due to the payer required for reallocating
    #[account(
        mut,
        realloc = SpaceShip::LEN + std::mem::size_of::<Module>() * (spaceship.modules.len() + 1) + std::mem::size_of::<Drone>() * (spaceship.drones.len() + 1) + std::mem::size_of::<Mutation>() * (spaceship.mutations.len() + 1),
        realloc::payer = user,
        realloc::zero = false,
        // seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), unknown index],
        // bump = spaceship.bump,
        constraint = user_account.spaceships.iter().map(|s|{s.spaceship}).collect::<Vec<_>>().contains(&spaceship.key()),
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

#[allow(unused_variables)] // due to #cfg[]
pub fn pick_crate(ctx: Context<PickCrate>, crate_type: CrateType) -> Result<()> {
    // cancel pending switchboard function request if stale
    {
        let spaceship = &mut ctx.accounts.spaceship;
        let current_slot = Realm::get_slot()?;
        if spaceship.crate_picking.switchboard_request_info.request_is_expired(current_slot) {
            spaceship.crate_picking.switchboard_request_info.status = SwitchboardFunctionRequestStatus::Expired { slot: current_slot  };
        }
    }

    // Validations
    {
        // verify that the user has an available crate
        require!(
            ctx.accounts.spaceship.experience.available_crate,
            HologramError::NoCrateAvailable
        );

        // verify that the user is not in the process of requesting to pick a crate already
        require!(
            !ctx.accounts.spaceship.crate_picking.switchboard_request_info.is_requested(),
            HologramError::CratePickingAlreadyRequested
        );
    }

    #[cfg(not(any(test, feature = "testing")))]
    {
        use switchboard_solana::{FunctionRequestSetConfig, FunctionRequestTrigger};

        let realm_key = ctx.accounts.realm.key();
        let user_account_seed = &[
            b"user_account",
            realm_key.as_ref(), ctx.accounts.user.key.as_ref(),
            &[ctx.accounts.user_account.bump],
        ];
        // Update the switchboard function parameters
        {


            let request_set_config_ctx = FunctionRequestSetConfig { request: ctx.accounts.switchboard_request.clone(), authority: ctx.accounts.admin.clone() };
            let request_params = format!(
                "PID={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={},CRATE_TYPE{}",
                crate::id(),
                ctx.accounts.user.key(),
                realm_key,
                ctx.accounts.user_account.key(),
                ctx.accounts.spaceship.key(),
                crate_type as u8,
            );

            request_set_config_ctx.invoke_signed(ctx.accounts.switchboard_program.clone(), request_params.into_bytes(), false, &[user_account_seed])?;
        }

        // Trigger the request account for the crate_picking_function
        // This will instruct the off-chain oracles to execute the docker container and relay
        // the result back to our program via the 'arena_matchmaking_settle' instruction.
        {
            let request_trigger_ctx = FunctionRequestTrigger {
                request: ctx.accounts.switchboard_request.clone(),
                authority: ctx.accounts.admin.clone(),
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
        }
    }

    // update spaceship crate_picking status
    {
        ctx.accounts
            .spaceship
            .crate_picking
            .switchboard_request_info
            .status = SwitchboardFunctionRequestStatus::Requested { slot: Realm::get_slot()?};
    }
    Ok(())
}

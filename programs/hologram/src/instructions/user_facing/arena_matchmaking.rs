use {
    crate::{
        error::HologramError,
        state::{
            Currency, MatchMakingStatus, Realm, SpaceShip, SpaceShipLite,
            SwitchboardFunctionRequestStatus, UserAccount,
        },
        ARENA_MATCHMAKING_FUEL_COST,
    },
    anchor_lang::prelude::*,
    switchboard_solana::{
        AttestationProgramState, AttestationQueueAccountData, FunctionAccountData,
        SWITCHBOARD_ATTESTATION_PROGRAM_ID,
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum Faction {
    Imperium = 1,
    Pirate,
    RogueDrone,
}

impl Faction {
    pub fn legal_tender(&self) -> Currency {
        match self {
            Faction::Imperium => Currency::ImperialCredit,
            Faction::Pirate => Currency::ImperialCredit,
            Faction::RogueDrone => Currency::ActivateNanitePaste,
        }
    }
}

#[derive(Accounts)]
#[instruction(spaceship_index:u8)]
pub struct ArenaMatchmaking<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Box<Account<'info, Realm>>,

    #[account(
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), spaceship_index.to_le_bytes().as_ref()],
        bump = spaceship.bump,
        constraint = spaceship.arena_matchmaking.switchboard_request_info.account == switchboard_request.key(),
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
        constraint = realm.switchboard_info.arena_matchmaking_function == arena_matchmaking_function.key() && !arena_matchmaking_function.load()?.requests_disabled
    )]
    pub arena_matchmaking_function: AccountLoader<'info, FunctionAccountData>,

    /// CHECK: in spaceship account's constraints
    #[account(mut)]
    pub switchboard_request: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub switchboard_request_escrow: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    /// CHECK: SWITCHBOARD_ATTESTATION_PROGRAM
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard_program: AccountInfo<'info>,
}

#[event]
pub struct ArenaMatchmakingQueueJoined {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

#[event]
pub struct ArenaMatchmakingMatching {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

#[event]
pub struct ArenaMatchmakingMatchingFailed {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

#[allow(unused_variables)]
pub fn arena_matchmaking(ctx: Context<ArenaMatchmaking>, faction: Faction) -> Result<()> {
    // cancel pending switchboard function request if stale
    {
        let spaceship = &mut ctx.accounts.spaceship;
        let current_slot = Realm::get_slot()?;
        if spaceship
            .arena_matchmaking
            .switchboard_request_info
            .request_is_expired(current_slot)
        {
            msg!("Matchmaking request is expired, cancelling");
            spaceship.arena_matchmaking.switchboard_request_info.status =
                SwitchboardFunctionRequestStatus::Expired { slot: current_slot };
            // update matchmaking status
            spaceship.arena_matchmaking.matchmaking_status = MatchMakingStatus::None;
            // refund fuel cost
            spaceship.fuel.refill(ARENA_MATCHMAKING_FUEL_COST)?;
            emit!(ArenaMatchmakingMatchingFailed {
                realm_name: ctx.accounts.realm.name.to_string(),
                user: ctx.accounts.user.key(),
                spaceship: SpaceShipLite::from_spaceship_account(spaceship)
            });
        }
    }

    // Validations
    {
        // verify that the user is not in the process of registering for the arena already
        require!(
            !ctx.accounts
                .spaceship
                .arena_matchmaking
                .switchboard_request_info
                .is_requested(),
            HologramError::ArenaMatchmakingAlreadyRequested
        );

        // verify that the user is not already in the queue
        require!(
            matches!(
                ctx.accounts.spaceship.arena_matchmaking.matchmaking_status,
                MatchMakingStatus::None
            ),
            HologramError::ArenaMatchmakingAlreadyInQueue
        );
    }

    // pay fuel entry price
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.fuel.consume(ARENA_MATCHMAKING_FUEL_COST)?;
    }

    // Matchmaking logic, two paths:
    // - the queue is filled, trigger match between the caller and a queue member
    // - the queue isn't filled, place the caller in the queue

    // @TODO: Will roll with this for now cause probably premature optimization, but there is a problem: If multiple users call this instruction and the queue is filled,
    //  they will all be matched with the same pool of opponent, which is limited, and the opponent is picked at random.
    //  There will be collisions, or lack of opponent depend of the amount of stress put on the system.
    //
    // Basically it's a concurrency issue, with a long async matching process, and a limited pool of opponent.
    //
    // Needs :
    // - The registration and matching need to stay decoupled, in order to avoid bundling TX and rerolling opponing
    // - Keep the matchmaking queue small if possible, to limit the amount of players waiting and also the on chain size of array
    //
    // Possible solution, from worst to best:
    // - A lock per queue. This would be a bottleneck, but would solve the issue.
    // - decoupling registration and matching fully. Downside is that we would need to store a big amount of opponents on chain.
    //    - a possible way to alleviate that could be to first call the matching IX through CPI when someone registers. Basically free up a space if possible first
    // - add a seed to MatchmakingQueue, and use it to pick an opponent. When a player registers and the queue is full,
    //   an opponent is selected base on the queue seed and the player seed (starting a match not for the caller but for the players already in the queue)
    //    - doesn't work cause the caller can bundle IX (and thus reroll opponent)
    //
    // Update: After long thinking... I think I got it. Ok so what we want to do is preshot the future, put your vypers and let me explain...
    //         We can add a counter in the MatchMaking Queue "requested_resolution" that we can increment right away, even if we don't know who is paired.
    //         When we pick the random opponent, we might rand over an already paired player, but that's ok, we just need to reroll or get the next one.
    //         Thanks to this we can reject if requested_resolution is >= max_spaceships in queue, that should give a more comfortable buffer before bottleneck.
    //         It's still britle, but with the different layers of Matchmaking, that we can extend, with the number of player per queue, that we can increase, with the different Realms, where we could have a player limit eventually.. Should work-ish?
    //
    {
        let spaceship = &mut ctx.accounts.spaceship;
        #[allow(unused_variables)] // due to #cfg[]
        let realm_key = ctx.accounts.realm.key();
        let realm = &mut ctx.accounts.realm;

        // find the queue matching spaceship Ordnance
        let queue = realm.get_matching_matchmaking_queue_mut(spaceship)?;

        // check that the system is not processing more matchmaking requests than there is spaceships in the queue (due to the concurrency issue described above)
        {
            require!(
                queue.matchmaking_request_count < queue.spaceships.len() as u8,
                HologramError::MatchmakingTooManyRequests
            );
        }

        // is the queue filled? Yes? -> matchmake, No? -> insert spaceship in the first available slot
        if queue.is_filled() {
            msg!("Matchmaking queue is filled, matching participants");
            // increase request awaiting settlement counter
            queue.matchmaking_request_count += 1;

            // Switchboard function bloc
            #[cfg(not(any(test, feature = "testing")))]
            {
                use {
                    crate::SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION,
                    switchboard_solana::{FunctionRequestSetConfig, FunctionRequestTrigger},
                };

                let user_account_seed = &[
                    b"user_account",
                    realm_key.as_ref(),
                    ctx.accounts.user.key.as_ref(),
                    &[ctx.accounts.user_account.bump],
                ];

                // Update the switchboard function parameters to include the queued spaceships
                {
                    let request_set_config_ctx = FunctionRequestSetConfig {
                        request: ctx.accounts.switchboard_request.clone(),
                        authority: ctx.accounts.user_account.to_account_info(),
                    };
                    let request_params = format!(
                        "PID={},USER={},REALM_PDA={},USER_ACCOUNT_PDA={},SPACESHIP_PDA={},FACTION={},OS_1_PDA={},OS_2_PDA={},OS_3_PDA={},OS_4_PDA={},OS_5_PDA={}",
                        crate::id(),
                        ctx.accounts.user.key(),
                        realm_key,
                        ctx.accounts.user_account.key(),
                        ctx.accounts.spaceship.key(),
                        faction as u8,
                        queue.spaceships[0].unwrap(),
                        queue.spaceships[1].unwrap(),
                        queue.spaceships[2].unwrap(),
                        queue.spaceships[3].unwrap(),
                        queue.spaceships[4].unwrap(),
                    );
                    msg!("{}", request_params);

                    request_set_config_ctx.invoke_signed(
                        ctx.accounts.switchboard_program.clone(),
                        request_params.into_bytes(),
                        false,
                        &[user_account_seed],
                    )?;
                    msg!("Switchboard function parameters updated");
                }

                // Trigger the request account for the arena_matchmaking_function
                // This will instruct the off-chain oracles to execute the docker container and relay
                // the result back to our program via the 'arena_matchmaking_settle' instruction.
                {
                    let request_trigger_ctx = FunctionRequestTrigger {
                        request: ctx.accounts.switchboard_request.clone(),
                        authority: ctx.accounts.user_account.to_account_info(),
                        escrow: ctx.accounts.switchboard_request_escrow.to_account_info(),
                        function: ctx.accounts.arena_matchmaking_function.to_account_info(),
                        state: ctx.accounts.switchboard_state.to_account_info(),
                        attestation_queue: ctx
                            .accounts
                            .switchboard_attestation_queue
                            .to_account_info(),
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

            // update arena_matchmaking status
            {
                let spaceship = &mut ctx.accounts.spaceship;
                spaceship.arena_matchmaking.switchboard_request_info.status =
                    SwitchboardFunctionRequestStatus::Requested {
                        slot: Realm::get_slot()?,
                    };
            }

            // update matchmaking status
            ctx.accounts.spaceship.arena_matchmaking.matchmaking_status =
                MatchMakingStatus::Matching {
                    slot: Realm::get_slot()?,
                };

            emit!(ArenaMatchmakingMatching {
                realm_name: ctx.accounts.realm.name.to_string(),
                user: ctx.accounts.user.key(),
                spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship)
            });
        } else {
            msg!("Matchmaking queue is not filled, adding spaceship to queue");
            // insert spaceship in the first available slot
            let empty_slot = queue.spaceships.iter_mut().find(|slot| slot.is_none());
            if let Some(slot) = empty_slot {
                *slot = Some(ctx.accounts.spaceship.key());
            } else {
                return Err(error!(HologramError::MatchmakingQueueFull)); // Should not happen as we checked the queue is not filled
            }

            // update matchmaking status
            ctx.accounts.spaceship.arena_matchmaking.matchmaking_status =
                MatchMakingStatus::InQueue {
                    slot: Realm::get_slot()?,
                };

            emit!(ArenaMatchmakingQueueJoined {
                realm_name: ctx.accounts.realm.name.to_string(),
                user: ctx.accounts.user.key(),
                spaceship: SpaceShipLite::from_spaceship_account(&ctx.accounts.spaceship)
            });
        }
    }

    Ok(())
}

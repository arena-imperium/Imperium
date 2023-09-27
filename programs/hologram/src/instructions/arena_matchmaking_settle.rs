use spaceship::{SwitchboardFunctionRequestStatus, MatchMakingStatus};

use crate::{utils::RandomNumberGenerator, ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE};

use {
    crate::{
        error::HologramError,
        state::{spaceship, Realm, SpaceShip, SpaceShipLite, UserAccount},
    },
    anchor_lang::prelude::*,
    switchboard_solana::prelude::*,
};

use solana_program::log::sol_log_compute_units;

#[derive(Accounts)]
pub struct ArenaMatchmakingSettle<'info> {

    #[account()]
    pub enclave_signer: Signer<'info>,
    
    /// CHECK: forwarded from the create_spaceship IX (and validated by it)
    #[account()]
    pub user: AccountInfo<'info>,

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

    #[account(
        mut,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), user_account.spaceships.len().to_le_bytes().as_ref()],
        bump = spaceship.bump,
        constraint = spaceship.randomness.switchboard_request_info.account == switchboard_request.key(),
    )]
    pub spaceship: Account<'info, SpaceShip>,

    #[account( 
        // validate that we use the realm custom switchboard function
        constraint = realm.switchboard_info.arena_matchmaking_function == arena_matchmaking_function.key()
    )]
    pub arena_matchmaking_function: AccountLoader<'info, FunctionAccountData>,

    #[account(
      constraint = switchboard_request.validate_signer(
          &arena_matchmaking_function.to_account_info(),
          &enclave_signer.to_account_info()
        )? @ HologramError::SwitchboardFunctionValidationFailed,
    )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,

    #[account(mut)]
    pub opponent_1_spaceship: Box<Account<'info, SpaceShip>>,

    #[account(mut)]
    pub opponent_2_spaceship: Box<Account<'info, SpaceShip>>,

    #[account(mut)]
    pub opponent_3_spaceship: Box<Account<'info, SpaceShip>>,

    #[account(mut)]
    pub opponent_4_spaceship: Box<Account<'info, SpaceShip>>,

    #[account(mut)]
    pub opponent_5_spaceship: Box<Account<'info, SpaceShip>>,
}

#[event]
pub struct ArenaMatchmakingMatchCompleted {
    pub realm_name: String,
    pub user: Pubkey,
    pub user_spaceship: SpaceShipLite,
    pub opponent_spaceship: SpaceShipLite,
}

#[event]
pub struct ArenaMatchmakingMatchExited {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

pub fn arena_matchmaking_settle(
    ctx: Context<ArenaMatchmakingSettle>,
    generated_seed: u32,
) -> Result<()> {
    // Validations
    {

        // verify that the request is pending settlement
        require!(
            ctx.accounts.spaceship.arena_matchmaking.switchboard_request_info.is_requested(),
            HologramError::ArenaMatchmakingAlreadySettled
        );

        // // verify that the switchboard request was successful
        // require!(
        //     ctx.accounts.switchboard_request.active_request.status == RequestStatus::RequestSuccess,
        //     HologramError::SwitchboardRequestNotSuccessful
        // );
    }

    // update caller arena_matchmaking status
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.arena_matchmaking.switchboard_request_info.status = SwitchboardFunctionRequestStatus::Settled { slot: Realm::get_slot()? };
        ctx.accounts.spaceship.arena_matchmaking.matchmaking_status = MatchMakingStatus::None;
    }

    // pick the opponent spaceship based on the random seed
    let opponent_spaceship = {
        let spaceship = &mut ctx.accounts.spaceship;
        let mut rng = RandomNumberGenerator::new(generated_seed.into());
        let dice_roll = rng.roll_dice(ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE as usize); // waiting for mem::variant_count::<Hull>() to be non nightly only rust...
        let queue = ctx.accounts.realm.get_matching_matchmaking_queue_mut(spaceship)?;
        let opponents_spaceship_keys = queue.spaceships.clone();

        // find the opponent spaceship account pubkey
        let mut opponent_spaceship_key: Pubkey = Pubkey::default();
        let dice_rolled_opponent_key = queue.spaceships.get((dice_roll - 1) as usize).unwrap();

        // if it was not found, pick the first spaceship available in the queue
        // this can happen due to the concurent nature of the program
        if let Some(key) = dice_rolled_opponent_key {
            opponent_spaceship_key = *key;
        } else {
            for spaceship_key in opponents_spaceship_keys {
                if let Some(key) = spaceship_key {
                    opponent_spaceship_key = key;
                    break;
                }
            }
        }
    
        // load the opponent spaceship based on the key        
        let opponent_spaceship = match opponent_spaceship_key {
            key if key == ctx.accounts.opponent_1_spaceship.key() => &mut ctx.accounts.opponent_1_spaceship,
            key if key == ctx.accounts.opponent_2_spaceship.key() => &mut ctx.accounts.opponent_2_spaceship,
            key if key == ctx.accounts.opponent_3_spaceship.key() => &mut ctx.accounts.opponent_3_spaceship,
            key if key == ctx.accounts.opponent_4_spaceship.key() => &mut ctx.accounts.opponent_4_spaceship,
            key if key == ctx.accounts.opponent_5_spaceship.key() => &mut ctx.accounts.opponent_5_spaceship,
            _ => panic!("Invalid spaceship key"),
        };

        // remove that spaceship from the matchmaking queue
        if let Some(spaceship) = queue.spaceships.iter_mut().find(|s| **s == Some(opponent_spaceship_key)) {
            *spaceship = None;
        }

        // updates the opponent matchmaking status
        opponent_spaceship.arena_matchmaking.matchmaking_status = MatchMakingStatus::None;

        opponent_spaceship
    };

    // FIGHT
    let (winner, looser) = {
        let spaceship = &mut ctx.accounts.spaceship;
        Realm::fight(spaceship, opponent_spaceship)
    };
    
    // distribute experience to participants
    {
        Realm::distribute_arena_experience(winner, looser);
    }

    // advance seeds
    {
        winner.randomness.advance_seed();
        looser.randomness.advance_seed();
    }

    // analytics
    {
        ctx.accounts.realm.analytics.total_arena_matches += 1;

        winner.analytics.total_arena_matches += 1;
        looser.analytics.total_arena_matches += 1;
        winner.analytics.total_arena_victories += 1;
    }

    sol_log_compute_units();

    Ok(())
}

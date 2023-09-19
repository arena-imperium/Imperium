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
    pub switchboard_request: Account<'info, FunctionRequestAccountData>,

    #[account(mut)]
    pub opponent_1_spaceship: Account<'info, SpaceShip>,

    #[account(mut)]
    pub opponent_2_spaceship: Account<'info, SpaceShip>,

    #[account(mut)]
    pub opponent_3_spaceship: Account<'info, SpaceShip>,

    #[account(mut)]
    pub opponent_4_spaceship: Account<'info, SpaceShip>,

    #[account(mut)]
    pub opponent_5_spaceship: Account<'info, SpaceShip>,
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
            matches!(ctx.accounts.spaceship.arena_matchmaking.switchboard_request_info.status, SwitchboardFunctionRequestStatus::Requested(_)),
            HologramError::SpaceshipRandomnessAlreadySettled
        );

        // // verify that the switchboard request was successful
        // require!(
        //     ctx.accounts.switchboard_request.active_request.status == RequestStatus::RequestSuccess,
        //     HologramError::SwitchboardRequestNotSuccessful
        // );
    }

    // update arena_matchmaking status
    {
        let spaceship = &mut ctx.accounts.spaceship;
        spaceship.arena_matchmaking.switchboard_request_info.status = SwitchboardFunctionRequestStatus::Settled(Realm::get_time()?);
        ctx.accounts.spaceship.arena_matchmaking.matchmaking_status = MatchMakingStatus::None;
    }

    // pick the opponent spaceship based on the random seed
    let (winner, looser) = {
        let spaceship = &mut ctx.accounts.spaceship;
        let mut rng = RandomNumberGenerator::new(generated_seed.into());
        let dice_roll = rng.roll_dice(ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE as usize); // waiting for mem::variant_count::<Hull>() to be non nightly only rust...
        let opponent_spaceship = match dice_roll {
            1 => &mut ctx.accounts.opponent_1_spaceship,
            2 => &mut ctx.accounts.opponent_2_spaceship,
            3 => &mut ctx.accounts.opponent_3_spaceship,
            4 => &mut ctx.accounts.opponent_4_spaceship,
            5 => &mut ctx.accounts.opponent_5_spaceship,
            _ => panic!("Invalid dice roll"),
        };

        // FIGHT
        {
            // emulate game engine for now
            let winner_roll = rng.roll_dice(2);
            match winner_roll {
                1 => (spaceship, opponent_spaceship),
                2 => (opponent_spaceship, spaceship),
                _ => panic!("Invalid dice roll"),
            }
        }
    };
    
    // distribute experience to participants
    {
        Realm::distribute_arena_experience(winner, looser);
    }

    // analytics
    {
        ctx.accounts.realm.analytics.total_arena_matches += 1;
    }

    Ok(())
}

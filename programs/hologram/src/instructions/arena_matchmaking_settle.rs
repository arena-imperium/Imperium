use spaceship::{SwitchboardFunctionRequestStatus, MatchMakingStatus};

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
    pub realm: Account<'info, Realm>,

    #[account(
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

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
    
    // remaining accounts contain the potential spaceship matches for matchmaking
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
    random_number: u8,
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





    Ok(())
}

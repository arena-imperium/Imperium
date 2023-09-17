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
pub struct CreateSpaceshipSettle<'info> {

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
        constraint = spaceship.randomness.switchboard_request == switchboard_request.key(),
    )]
    pub spaceship: Account<'info, SpaceShip>,

    #[account( 
        // validate that we use the realm custom switchboard function
        constraint = realm.switchboard_info.spaceship_seed_generation_function == switchboard_function.key()
    )]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,

    #[account(
      constraint = switchboard_request.validate_signer(
          &switchboard_function.to_account_info(),
          &enclave_signer.to_account_info()
        )? @ HologramError::SwitchboardFunctionValidationFailed,
    )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
}

#[event]
pub struct SpaceshipCreated {
    pub realm_name: String,
    pub user: Pubkey,
    pub spaceship: SpaceShipLite,
}

pub fn create_spaceship_settle(
    ctx: Context<CreateSpaceshipSettle>,
    generated_seed: u32,
) -> Result<()> {
    // Validations
    {
        // verify that this request was not settled already
        require!(
            ctx.accounts.spaceship.randomness.status == spaceship::SwitchboardFunctionRequestStatus::Requested,
            HologramError::SpaceshipRandomnessAlreadySettled
        );

        // // verify that the switchboard request was successful
        // require!(
        //     ctx.accounts.switchboard_request.active_request.status == RequestStatus::RequestSuccess,
        //     HologramError::SwitchboardRequestNotSuccessful
        // );
    }

    // Finish Spaceship initialization with the generated seed
    {
        ctx.accounts.spaceship.randomness.status = spaceship::SwitchboardFunctionRequestStatus::Settled;
        ctx.accounts.spaceship.randomness.original_seed = generated_seed.into();
        ctx.accounts.spaceship.randomness.current_seed = generated_seed.into();
        ctx.accounts.spaceship.randomness.iteration = 1;
    }

    // Roll the Hull with the first generated seed
    {
        let mut rng = RandomNumberGenerator::new(generated_seed.into());
        let dice_roll = rng.roll_dice(10); // waiting for mem::variant_count::<Hull>() to be non nightly only rust...
        ctx.accounts.spaceship.hull = match dice_roll {
            1 => Hull::CommonOne,
            2 => Hull::CommonTwo,
            3 => Hull::CommonThree,
            4 => Hull::UncommonOne,
            5 => Hull::UncommonTwo,
            6 => Hull::UncommonThree,
            7 => Hull::UncommonFour,
            8 => Hull::RareOne,
            9 => Hull::RareTwo,
            10 => Hull::MythicalOne,
            _ => panic!("Invalid dice roll"),
        };
    }

    // Update realm stats
    {
        ctx.accounts.realm.stats.total_spaceships_created += 1;
    }

    emit!(SpaceshipCreated {
        realm_name: ctx.accounts.realm.name.to_string(),
        user: ctx.accounts.user.key(),
        spaceship: SpaceShipLite {
            name: ctx.accounts.spaceship.name,
            hull: ctx.accounts.spaceship.hull.clone(),
            spaceship: *ctx.accounts.spaceship.to_account_info().key,
        },
    });
    Ok(())
}

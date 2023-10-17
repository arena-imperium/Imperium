#[allow(unused_imports)]
use switchboard_solana::FunctionRequestAccountData;
use {
    crate::{
        engine::LT_STARTER_OFFENSIVE_MODULES,
        error::HologramError,
        state::{
            spaceship, Currency, Realm, SpaceShip, SpaceShipLite, SwitchboardFunctionRequestStatus,
            UserAccount,
        },
        utils::RandomNumberGenerator,
        STARTING_IMPERIAL_CREDITS,
    },
    anchor_lang::prelude::*,
    spaceship::Hull,
    switchboard_solana::{self, FunctionAccountData},
};

#[derive(Accounts)]
pub struct CreateSpaceshipSettle<'info> {
    #[account()]
    pub enclave_signer: Signer<'info>,

    /// CHECK: forwarded from the create_spaceship IX (and validated by it)
    #[account()]
    pub user: AccountInfo<'info>,

    #[account(
        mut,
        seeds=[b"realm", realm.name.to_bytes()],
        bump = realm.bump,
    )]
    pub realm: Account<'info, Realm>,

    #[account(
        mut,
        seeds=[b"user_account", realm.key().as_ref(), user.key.as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        seeds=[b"spaceship", realm.key().as_ref(), user.key.as_ref(), (user_account.spaceships.len() as u8).to_le_bytes().as_ref()],
        bump = spaceship.bump,
        constraint = spaceship.randomness.switchboard_request_info.account == switchboard_request.key(),
    )]
    pub spaceship: Account<'info, SpaceShip>,

    #[account(
        // validate that we use the realm custom switchboard function
        constraint = realm.switchboard_info.spaceship_seed_generation_function == spaceship_seed_generation_function.key()
    )]
    pub spaceship_seed_generation_function: AccountLoader<'info, FunctionAccountData>,

    #[cfg(not(any(test, feature = "testing")))]
    #[account(
        // validation of the signer is done in the IX code
    )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
    #[cfg(any(test, feature = "testing"))]
    /// CHECK: test target only
    pub switchboard_request: AccountInfo<'info>,
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
        // verify that the call was made by the container
        // Disabled during tests
        #[cfg(not(any(test, feature = "testing")))]
        require!(
            ctx.accounts.switchboard_request.validate_signer(
                &ctx.accounts
                    .spaceship_seed_generation_function
                    .to_account_info(),
                &ctx.accounts.enclave_signer.to_account_info()
            ) == Ok(true),
            HologramError::FunctionValidationFailed
        );

        // verify that the request is pending settlement
        require!(
            ctx.accounts
                .spaceship
                .randomness
                .switchboard_request_info
                .is_requested(),
            HologramError::SpaceshipRandomnessAlreadySettled
        );

        // // verify that the switchboard request was successful
        // #[cfg(not(any(test, feature = "testing")))]
        // require!(
        //     ctx.accounts.switchboard_request.active_request.status == RequestStatus::RequestSuccess,
        //     HologramError::SwitchboardRequestNotSuccessful
        // );
    }

    // Finish Spaceship initialization with the generated seed
    {
        ctx.accounts
            .spaceship
            .randomness
            .switchboard_request_info
            .status = SwitchboardFunctionRequestStatus::Settled {
            slot: Realm::get_slot()?,
        };
        ctx.accounts.spaceship.randomness.original_seed = generated_seed.into();
        ctx.accounts.spaceship.randomness.current_seed = generated_seed.into();
        ctx.accounts.spaceship.randomness.iteration = 1;
    }

    let mut rng = RandomNumberGenerator::new(generated_seed.into());

    // Roll the Hull with the first generated seed
    {
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
            10 => Hull::FactionOne,
            _ => panic!("Invalid dice roll"),
        };
    }

    // provide spaceship with starting module and credits
    {
        let spaceship = &mut ctx.accounts.spaceship;
        // provide starter offensive_module
        {
            let roll = rng.roll_dice(LT_STARTER_OFFENSIVE_MODULES.len());
            let starter_offensive_module = LT_STARTER_OFFENSIVE_MODULES[roll as usize - 1].clone();
            msg!("Starter offensive module: {:?}", starter_offensive_module);
            spaceship.mount_module(starter_offensive_module)?;
        }
        // provide starting imperial credits
        {
            spaceship
                .wallet
                .credit(STARTING_IMPERIAL_CREDITS, Currency::ImperialCredit)?;
        }
    }

    // set unique spaceship ID
    {
        msg!(
            "Total spaceships created: {}",
            ctx.accounts.realm.analytics.total_spaceships_created
        );
        ctx.accounts.spaceship.id = ctx.accounts.realm.analytics.total_spaceships_created;
    }

    let spaceship_lite = SpaceShipLite {
        name: ctx.accounts.spaceship.name,
        hull: ctx.accounts.spaceship.hull.clone(),
        spaceship: *ctx.accounts.spaceship.to_account_info().key,
    };

    // Create spaceship reference in user_account
    {
        let user_account = &mut ctx.accounts.user_account;
        user_account.spaceships.push(spaceship_lite.clone());
    }

    // Update realm analytics
    {
        ctx.accounts.realm.analytics.total_spaceships_created += 1;
    }

    emit!(SpaceshipCreated {
        realm_name: ctx.accounts.realm.name.to_string(),
        user: ctx.accounts.user.key(),
        spaceship: spaceship_lite,
    });
    Ok(())
}

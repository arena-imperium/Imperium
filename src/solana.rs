pub use anchor_client::Client as AnchorClient;
use {
    anchor_client::{
        anchor_lang::{prelude::System, Id},
        ClientError, Cluster, Program,
    },
    bevy::{
        log,
        prelude::{App, Commands, Component, Entity, Plugin, PostUpdate, Query, Resource},
        tasks::{IoTaskPool, Task},
    },
    futures_lite::future,
    hologram::{
        self,
        instructions::{CrateType, Faction},
        state::{SpaceShip, UserAccount},
    },
    solana_cli_output::display::println_transaction,
    solana_client::{rpc_config::RpcTransactionConfig, rpc_filter::RpcFilterType},
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
    },
    solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding},
    spl_associated_token_account::get_associated_token_address,
    std::{any::type_name, env, fmt, str::FromStr, sync::Arc},
    switchboard_solana::{
        anchor_spl::token::spl_token::native_mint, AccountDeserialize, Discriminator,
    },
};

pub struct SolanaPlugin;

impl Plugin for SolanaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HologramServer>()
            .add_systems(PostUpdate, solana_transaction_task_handler);
    }
}

#[derive(Component)]
pub struct SolanaTransactionTask {
    pub description: String,
    pub task: Task<Result<EncodedConfirmedTransactionWithStatusMeta, SolanaTransactionTaskError>>,
}

pub(crate) fn solana_transaction_task_handler(
    mut commands: Commands,
    mut solana_transaction_tasks: Query<(Entity, &mut SolanaTransactionTask)>,
) {
    for (entity, mut task) in &mut solana_transaction_tasks {
        match future::block_on(future::poll_once(&mut task.task)) {
            Some(result) => {
                let status = match result {
                    Ok(confirmed_transaction) => {
                        match confirmed_transaction.transaction.meta.unwrap().err {
                            Some(error) => {
                                format!("Transaction failed: {}", error)
                            }
                            None => "Transaction succeeded".to_string(),
                        }
                    }
                    Err(error) => {
                        format!("Transaction failed: {}", error)
                    }
                };
                let message = format!("{}: {}", task.description, status);
                log::info!("{}", message);
                commands.entity(entity).despawn();
            }
            None => {}
        };
    }
}

pub enum SolanaTransactionTaskError {
    SolanaClientError(solana_client::client_error::ClientError),
    AnchorClientError(anchor_client::ClientError),
}

impl From<anchor_client::ClientError> for SolanaTransactionTaskError {
    fn from(error: anchor_client::ClientError) -> Self {
        SolanaTransactionTaskError::AnchorClientError(error)
    }
}

impl fmt::Display for SolanaTransactionTaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SolanaTransactionTaskError::SolanaClientError(e) => {
                write!(f, "SolanaClientError: {:?}", e)
            }
            SolanaTransactionTaskError::AnchorClientError(e) => {
                write!(f, "AnchorClientError: {:?}", e)
            }
        }
    }
}

#[derive(Component)]
pub struct SolanaFetchAccountTask<T> {
    pub description: String,
    pub task: Task<Result<T, ClientError>>,
}

#[derive(Component)]
pub struct SolanaFetchAccountsTask<T> {
    pub description: String,
    pub task: Task<Result<Vec<(Pubkey, T)>, ClientError>>,
}

#[derive(Resource)]
pub struct HologramServer {
    pub solana_client: Arc<SolanaClient>,
    pub realm_name: String,
    pub admin_pubkey: Pubkey,
    // Custom Switchboard functions
    pub spaceship_seed_generation_function: Pubkey,
    pub arena_matchmaking_function: Pubkey,
    pub crate_picking_function: Pubkey,
    // Switchboard attestation queue
    pub switchboard_attestation_queue: Pubkey,
}

impl Default for HologramServer {
    fn default() -> HologramServer {
        dotenv::dotenv().ok();
        // Solana setup
        let payer = match env::var("SOLANA_PAYER_KEY").ok() {
            Some(k) => read_keypair_file(&*shellexpand::tilde(&k))
                .expect("Failed to parse $SOLANA_PAYER_KEY"),
            None => panic!("Could not load payer key,"),
        };
        let rpc_url = match env::var("SOLANA_RPC_URL").ok() {
            Some(url) => url,
            None => panic!("Could not load solana_rpc_url,"),
        };
        let ws_url = match env::var("SOLANA_WS_URL").ok() {
            Some(url) => url,
            None => panic!("Could not load solana_ws_url,"),
        };

        let cluster = Cluster::Custom(rpc_url, ws_url);
        let client = Arc::new(SolanaClient::new(payer.clone(), cluster));
        HologramServer {
            solana_client: client,
            realm_name: "Holorealm1".to_string(), // @HARDCODED
            admin_pubkey: Pubkey::from_str("A4PzGUimdCMv8xvT5gK2fxonXqMMayDm3eSXRvXZhjzU").unwrap(),
            spaceship_seed_generation_function: Pubkey::from_str(
                "5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe",
            )
            .unwrap(), // @HARDCODED
            arena_matchmaking_function: Pubkey::from_str(
                "4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL",
            )
            .unwrap(), // @HARDCODED
            crate_picking_function: Pubkey::from_str(
                "EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG",
            )
            .unwrap(), // @HARDCODED
            switchboard_attestation_queue: Pubkey::from_str(
                "CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE", // @HARDCODED @TODO
            )
            .unwrap(),
        }
    }
}

// DEVNET test spaceship seed generation function 5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe
// DEVNET test arena matchmaking function 4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL
// DEVNET test crate picking function EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG
// https://app.switchboard.xyz/build/function/5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe
// https://app.switchboard.xyz/build/function/4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL
// https://app.switchboard.xyz/build/function/EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG

// See anchor client blocking sources https://github.com/coral-xyz/anchor/blob/cec9946111a1c651fd21235c2a554eda05c3ffa3/client/src/blocking.rs
// and examples https://github.com/coral-xyz/anchor/blob/cec9946111a1c651fd21235c2a554eda05c3ffa3/client/example/src/blocking.rs

impl HologramServer {
    // THESE DEFAULT SHOULD not be here. Better have the debug UI provide field to fill in the parameters at the top level and use these
    pub fn fire_default_initialize_realm_task(&self, commands: &mut Commands) {
        self.fire_initialize_realm_task(
            commands,
            &self.realm_name,
            &self.spaceship_seed_generation_function,
            &self.arena_matchmaking_function,
            &self.crate_picking_function,
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn fire_default_create_user_account_task(&self, commands: &mut Commands) {
        let (realm_pda, _) = Self::get_realm_pda(&self.realm_name);
        self.fire_create_user_account_task(
            commands,
            &realm_pda,
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn fire_default_create_spaceship_task(&self, commands: &mut Commands) {
        let (realm_pda, _) = Self::get_realm_pda(&self.realm_name);
        self.fire_create_spaceship_task(
            commands,
            &"Nebuchadnezzar".to_string(),
            &realm_pda,
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn fire_default_pick_crate_task(&self, commands: &mut Commands) {
        let (realm_pda, _) = Self::get_realm_pda(&self.realm_name);
        let user = &self.solana_client.payer.pubkey();
        let spaceship_index = 0;
        self.fire_pick_crate_task(
            commands,
            &realm_pda,
            &user,
            spaceship_index,
            CrateType::NavyIssue,
        );
    }

    pub fn fire_default_arena_matchmaking_task(&self, commands: &mut Commands) {
        let (realm_pda, _) = Self::get_realm_pda(&self.realm_name);
        let user = &self.solana_client.payer.pubkey();
        let spaceship_index = 0;
        self.fire_arena_matchmaking_task(
            commands,
            &realm_pda,
            &user,
            spaceship_index,
            Faction::Imperium,
        );
    }

    /// This function fires an initialize realm task.
    /// This will create a new realm account and set the admin to `admin`.
    ///
    /// Anchor events:
    /// - RealmInitialized: inform that the realm was created
    ///
    /// Parameters:
    /// commands: Commands
    /// realm_name: name of the realm to create
    /// spaceship_seed_generation_function: Pubkey of the switchboard function to use for spaceship seed generation
    /// arena_matchmaking_function: Pubkey of the switchboard function to use for arena matchmaking
    /// crate_picking_function: Pubkey of the switchboard function to use for crate picking
    /// admin: Pubkey of the admin of the realm
    pub fn fire_initialize_realm_task(
        &self,
        commands: &mut Commands,
        realm_name: &String,
        spaceship_seed_generation_function: &Pubkey,
        arena_matchmaking_function: &Pubkey,
        crate_picking_function: &Pubkey,
        admin: &Pubkey, // Here should be a keypair, but it's just the payer. This IX is not really meant to be in this bevy app (as it's more of an admin IX), just temporary for dev
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let payer = client.payer().clone();
        let admin = admin.clone();
        let realm_name = realm_name.clone();
        let spaceship_seed_generation_function = spaceship_seed_generation_function.clone();
        let arena_matchmaking_function = arena_matchmaking_function.clone();
        let crate_picking_function = crate_picking_function.clone();

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending initialize_realm IX");
            let (realm_pda, _) = Self::get_realm_pda(&realm_name);

            let instruction = hologram::instruction::InitializeRealm { name: realm_name };
            let accounts = hologram::accounts::InitializeRealm {
                payer: payer.pubkey(),
                admin,
                realm: realm_pda,
                spaceship_seed_generation_function,
                arena_matchmaking_function,
                crate_picking_function,
                system_program: solana_program::system_program::id(),
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer.clone(),
                vec![],
                200_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "initialize_realm".to_string(),
            task,
        });
    }

    /// This function fires a create user account task.
    /// This will create a new user account in the realm for the provided user (only one per user per realm (wallet))
    ///
    /// Anchor events:
    /// - UserAccountCreated: inform that the user account was created
    ///
    /// Parameters:
    /// commands: Commands
    /// realm_name: name of the realm to create the account for
    /// user: Pubkey
    pub fn fire_create_user_account_task(
        &self,
        commands: &mut Commands,
        realm_pda: &Pubkey,
        user: &Pubkey,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let user = user.clone();
        let payer = client.payer().clone();
        let realm_pda = realm_pda.clone();

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending create_user_account IX");
            let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, &user);

            let instruction = hologram::instruction::CreateUserAccount {};
            let accounts = hologram::accounts::CreateUserAccount {
                user,
                realm: realm_pda,
                user_account: user_account_pda,
                system_program: solana_program::system_program::id(),
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer,
                vec![],
                200_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "create_user_account".to_string(),
            task,
        });
    }

    /// This function fires a create spaceship task.
    /// This will add a new spaceship to the user account (max 10).
    ///
    /// Anchor events:
    /// - SpaceshipCreated: inform that the spaceship was created
    ///
    /// Parameters:
    /// commands: Commands
    /// spaceship_name: String (max 64 chars)
    /// user: The user making the call (user_account is derived)
    pub fn fire_create_spaceship_task(
        &self,
        commands: &mut Commands,
        spaceship_name: &String,
        realm_pda: &Pubkey,
        user: &Pubkey,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let admin_pubkey = self.admin_pubkey;
        let realm_pda = realm_pda.clone();
        let user = user.clone();
        let spaceship_seed_generation_function = self.spaceship_seed_generation_function;
        let arena_matchmaking_function = self.arena_matchmaking_function;
        let crate_picking_function = self.crate_picking_function;
        let spaceship_name = spaceship_name.clone();
        let payer = client.payer().clone();
        let switchboard_attestation_queue = self.switchboard_attestation_queue;

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending create_spaceship IX");
            let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, &user);

            // @TODO use cache
            let user_account: UserAccount = client
                .anchor_client
                .program(hologram::id())?
                .account(user_account_pda)?;
            let new_spaceship_index = user_account.spaceships.len();

            let (spaceship_pda, _) =
                Self::get_spaceship_pda(&realm_pda, &user, new_spaceship_index as u8);
            let (switchboard_state_pda, _) = Self::get_switchboard_state();
            let switchboard_ssgf_request_keypair = Keypair::new();
            let switchboard_ssgf_request_escrow = get_associated_token_address(
                &switchboard_ssgf_request_keypair.pubkey(),
                &native_mint::ID,
            );
            let switchboard_amf_request_keypair = Keypair::new();
            let switchboard_amf_request_escrow = get_associated_token_address(
                &switchboard_amf_request_keypair.pubkey(),
                &native_mint::ID,
            );
            let switchboard_cpf_request_keypair = Keypair::new();
            let switchboard_cpf_request_escrow = get_associated_token_address(
                &switchboard_cpf_request_keypair.pubkey(),
                &native_mint::ID,
            );
            let instruction = hologram::instruction::CreateSpaceship {
                name: spaceship_name,
            };

            let accounts = hologram::accounts::CreateSpaceship {
                user,
                realm: realm_pda,
                admin: admin_pubkey,
                user_account: user_account_pda,
                spaceship: spaceship_pda,
                switchboard_state: switchboard_state_pda,
                switchboard_attestation_queue,
                spaceship_seed_generation_function,
                switchboard_ssgf_request: switchboard_ssgf_request_keypair.pubkey(),
                switchboard_ssgf_request_escrow,
                arena_matchmaking_function,
                switchboard_amf_request: switchboard_amf_request_keypair.pubkey(),
                switchboard_amf_request_escrow,
                crate_picking_function,
                switchboard_cpf_request: switchboard_cpf_request_keypair.pubkey(),
                switchboard_cpf_request_escrow,
                switchboard_mint: native_mint::ID,
                system_program: solana_program::system_program::id(),
                token_program: switchboard_solana::anchor_spl::token::ID,
                switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
                associated_token_program: switchboard_solana::anchor_spl::associated_token::ID,
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer,
                vec![
                    switchboard_ssgf_request_keypair,
                    switchboard_amf_request_keypair,
                    switchboard_cpf_request_keypair,
                ],
                650_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "create_spaceship".to_string(),
            task,
        });
    }

    /// This function fires a claim_fuel_allowance task.
    /// This will attempt to get the periodical free fuel allowance for a given spaceship.
    ///
    ///  Anchor events:
    /// - FuelAllowanceClaimed: inform that the fuel allowance was claimed
    ///
    /// Parameters:
    /// commands: Commands
    /// spaceship_pda: Pubkey of the spaceship to claim the fuel allowance for
    /// user: The user making the call (user_account is derived)
    pub fn fire_claim_fuel_allowance_task(
        &self,
        commands: &mut Commands,
        realm_pda: &Pubkey,
        user: &Pubkey,
        spaceship_index: u8,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let realm_pda = realm_pda.clone();
        let user = user.clone();
        let payer = client.payer().clone();
        let (spaceship_pda, _) = Self::get_spaceship_pda(&realm_pda, &user, spaceship_index.into());

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending claim_fuel_allowance IX");
            let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, &user);
            let instruction = hologram::instruction::ClaimFuelAllowance { spaceship_index };

            let accounts = hologram::accounts::ClaimFuelAllowance {
                user,
                realm: realm_pda,
                user_account: user_account_pda,
                spaceship: spaceship_pda,
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer,
                vec![],
                50_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "claim_fuel_allowance".to_string(),
            task,
        });
    }

    /// This function fires an arena matchmaking task.
    /// This will register the spaceship in the matchmaking queue.
    /// It triggers an offchain SBf() for the settlement callback. There is a 30 second timeout.
    ///
    /// Anchor events:
    /// - ArenaMatchmakingQueueJoined: user has joined the queue and is waiting for more participants to join in in order to trigger a match
    /// - ArenaMatchmakingQueueMatching: user is being matched with a queue participant as the queue is full (call to the async SBf() function)
    /// - ArenaMatchmakingQueueMatchingFailed: not sent automatically, will be emited if the user re register for the queue after a timeout (30 seconds)
    /// - ArenaMatchmakingMatchCompleted: the players have been matched, a winner has been decided. Return info about winner/looser
    ///
    /// Parameters:
    /// - commands: Command buffer to execute the task.
    /// - user: Public key of the user owning the spaceship
    /// - spaceship_pda: Public key of the spaceship that registers
    /// - faction: Faction where the fight takes place, will determine the rewards.
    pub fn fire_arena_matchmaking_task(
        &self,
        commands: &mut Commands,
        realm_pda: &Pubkey,
        user: &Pubkey,
        spaceship_index: u8,
        // Location
        faction: Faction,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let arena_matchmaking_function = self.arena_matchmaking_function;
        let realm_pda = realm_pda.clone();
        let user = user.clone();
        let (spaceship_pda, _) = Self::get_spaceship_pda(&realm_pda, &user, spaceship_index.into());
        let payer = client.payer().clone();
        let switchboard_attestation_queue = self.switchboard_attestation_queue;

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending arena_matchmaking IX");
            let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, &user);
            let (switchboard_state_pda, _) = Self::get_switchboard_state();
            // retrieve the switchboard_amf_request from spaceship (@TODO use cache?)
            let spaceship: SpaceShip = client
                .anchor_client
                .program(hologram::id())?
                .account(spaceship_pda)?;

            let switchboard_amf_request =
                spaceship.arena_matchmaking.switchboard_request_info.account;
            let switchboard_amf_request_escrow =
                get_associated_token_address(&switchboard_amf_request, &native_mint::ID);
            let instruction = hologram::instruction::ArenaMatchmaking {
                faction,
                spaceship_index,
            };

            let accounts = hologram::accounts::ArenaMatchmaking {
                user,
                realm: realm_pda,
                user_account: user_account_pda,
                spaceship: spaceship_pda,
                switchboard_state: switchboard_state_pda,
                switchboard_attestation_queue,
                arena_matchmaking_function,
                switchboard_request: switchboard_amf_request,
                switchboard_request_escrow: switchboard_amf_request_escrow,
                system_program: solana_program::system_program::id(),
                token_program: switchboard_solana::anchor_spl::token::ID,
                switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer,
                vec![],
                250_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "arena_matchmaking".to_string(),
            task,
        });
    }

    /// This function fires a crate picking task.
    ///
    /// Anchor events:
    /// - CratePicked: user has picked a crate and received the reward
    /// - CratePickingFailed: user has failed to pick a crate (no more crates available)
    /// - CratePickingFailed: user has failed to pick a crate (no more crates available)
    ///
    /// Parameters:
    /// - commands: Command buffer to execute the task.
    /// - user: Public key of the user owning the spaceship
    /// - spaceship_pda: Public key of the spaceship that registers
    /// - crate_type: The type of crate to pick
    fn fire_pick_crate_task(
        &self,
        commands: &mut Commands,
        realm_pda: &Pubkey,
        user: &Pubkey,
        spaceship_index: u8,
        // Location
        crate_type: CrateType,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let crate_picking_function = self.crate_picking_function;
        let realm_pda = realm_pda.clone();
        let user = user.clone();
        let (spaceship_pda, _) = Self::get_spaceship_pda(&realm_pda, &user, spaceship_index.into());
        let payer = client.payer().clone();
        let switchboard_attestation_queue = self.switchboard_attestation_queue;

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Sending pick_crate IX");
            let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, &user);
            let (switchboard_state_pda, _) = Self::get_switchboard_state();
            // retrieve the switchboard_cpf_request from spaceship (@TODO use cache?)
            let spaceship: SpaceShip = client
                .anchor_client
                .program(hologram::id())?
                .account(spaceship_pda)?;

            let switchboard_cpf_request = spaceship.crate_picking.switchboard_request_info.account;
            let switchboard_cpf_request_escrow =
                get_associated_token_address(&switchboard_cpf_request, &native_mint::ID);
            let instruction = hologram::instruction::PickCrate {
                crate_type,
                spaceship_index,
            };

            let accounts = hologram::accounts::PickCrate {
                user,
                realm: realm_pda,
                user_account: user_account_pda,
                spaceship: spaceship_pda,
                switchboard_state: switchboard_state_pda,
                switchboard_attestation_queue,
                crate_picking_function,
                switchboard_request: switchboard_cpf_request,
                switchboard_request_escrow: switchboard_cpf_request_escrow,
                system_program: solana_program::system_program::id(),
                token_program: switchboard_solana::anchor_spl::token::ID,
                switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
            };

            Self::send_and_confirm_instruction_blocking(
                client,
                hologram::id(),
                instruction,
                accounts,
                payer,
                vec![],
                250_000,
            )
        });

        commands.spawn(SolanaTransactionTask {
            description: "pick_crate".to_string(),
            task,
        });
    }

    /// Returns the account at the given address
    fn fire_fetch_account_task<T: 'static + AccountDeserialize + Send>(
        &self,
        commands: &mut Commands,
        account: &Pubkey,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);
        let account_clone = account.clone();

        let task = thread_pool.spawn(async move {
            log::info!("<Solana> Fetching account {}", account_clone);
            let account: T = client
                .anchor_client
                .program(hologram::id())?
                .account(account_clone)?;
            Ok(account)
        });

        commands.spawn(SolanaFetchAccountTask {
            description: "fetch_account".to_string(),
            task,
        });
    }

    /// Returns all program accounts of the given type matching the given filters
    fn fire_fetch_accounts_with_filter_task<
        T: 'static + AccountDeserialize + Discriminator + Send,
    >(
        &self,
        commands: &mut Commands,
        filters: Vec<RpcFilterType>,
    ) {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);

        let task = thread_pool.spawn(async move {
            log::info!(
                "<Solana> Fetching accounts of type {} applying provided filter",
                type_name::<T>()
            );
            let account: Vec<(Pubkey, T)> = client
                .anchor_client
                .program(hologram::id())?
                .accounts(filters)?;
            Ok(account)
        });

        commands.spawn(SolanaFetchAccountsTask {
            description: "fetch_accounts_with_filter".to_string(),
            task,
        });
    }

    // pub async fn events<C: Deref<Target = impl Signer> + Clone>(&self, pid: Pubkey) -> Result<()> {
    //     let client = Arc::clone(&self.solana_client);
    //     let program = client.program(pid)?;

    //     let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    //     let event_unsubscriber = program
    //         .on(move |_, event: MyEvent| {
    //             if sender.send(event).is_err() {
    //                 println!("Error while transferring the event.")
    //             }
    //         })
    //         .await?;

    //     let event = receiver.recv().await.unwrap();
    //     assert_eq!(event.data, 5);
    //     assert_eq!(event.label, "hello".to_string());

    //     Ok(())
    // }

    /// Sends and confirms an instruction to the Solana cluster
    fn send_and_confirm_instruction_blocking(
        client: Arc<SolanaClient>,
        program_id: Pubkey,
        instruction: impl anchor_client::anchor_lang::InstructionData + Send + 'static,
        accounts: impl anchor_client::anchor_lang::ToAccountMetas + Send + 'static,
        payer: Keypair,
        additionnal_signers: Vec<Keypair>,
        compute_budget_limit: u32,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta, SolanaTransactionTaskError> {
        let program = client.anchor_client.program(program_id).unwrap();

        let increase_compute_budget_ix =
            solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(
                compute_budget_limit,
            );
        let mut request = program
            .request()
            .instruction(increase_compute_budget_ix)
            .args(instruction)
            .signer(&payer)
            .accounts(accounts);

        let cloned_signers = additionnal_signers
            .iter()
            .map(|signer| signer.clone())
            .collect::<Vec<_>>();
        for signer in &cloned_signers {
            request = request.signer(signer);
        }

        let result = request.send();
        match result {
            Ok(tx) => {
                let rpc_client = client.anchor_client.program(System::id())?.rpc();
                let result = rpc_client.get_transaction_with_config(
                    &tx,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Binary),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: None,
                    },
                );
                match result {
                    Ok(tx) => {
                        log::info!("Transaction confirmed");
                        Ok(tx)
                    }
                    Err(e) => Err(SolanaTransactionTaskError::SolanaClientError(e)),
                }
            }
            Err(e) => Err(SolanaTransactionTaskError::AnchorClientError(e)),
        }
    }

    pub fn get_realm_pda(realm_name: &String) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"realm", realm_name.as_bytes()], &hologram::id())
    }

    pub fn get_user_account_pda(realm_pda: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"user_account", realm_pda.as_ref(), user.as_ref()],
            &hologram::id(),
        )
    }

    // FLOW
    // 1. You know the real name, let's say it's "GameRealm1" (alluser share this realm)
    // 2. You know the user's pubkey, that's the one you sign with
    // 3. From this user pubkey, you can derive it's UserAccount PDA (because a UserAccount PDA == "user_account" + realm_pda + user_pubkey)
    // 4. Check if the UserAccount exists 
    // 4.1. if not, create it (calling the create_user_account IX)
    // 4.2. if yes, continue
    // 5. you now have a user_account PDA (the address), so you can call fetch_account in solana.rs with the user_account PDA to retreive the data,
    //    that data contains the list of spaceship PDAs
  

    pub fn get_spaceship_pda(
        realm_pda: &Pubkey,
        user: &Pubkey,
        spaceship_index: u8,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"spaceship",
                realm_pda.as_ref(),
                user.as_ref(),
                spaceship_index.to_le_bytes().as_ref(),
            ],
            &hologram::id(),
        )
    }

    pub fn get_switchboard_state() -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[switchboard_solana::STATE_SEED],
            &switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
        )
    }
}

// -----------------------------------------------------------------------------

type Payer = Arc<Keypair>;

/// `Client` allows you to send typed RPC requests to a Solana cluster.
pub struct SolanaClient {
    payer: Payer,
    anchor_client: AnchorClient<Payer>,
}
#[allow(dead_code)]
impl SolanaClient {
    /// Creates a new `Client` instance.
    pub fn new(payer: Keypair, cluster: Cluster) -> Self {
        let payer = Arc::new(payer);
        Self {
            payer: payer.clone(),
            anchor_client: AnchorClient::new_with_options(
                cluster,
                payer,
                CommitmentConfig::confirmed(), // TODO update
            ),
        }
    }

    /// Gets client's payer.
    pub fn payer(&self) -> &Keypair {
        &self.payer
    }

    /// Gets the internal Anchor client to call Anchor client's methods directly.
    pub fn anchor_client(&self) -> &AnchorClient<Payer> {
        &self.anchor_client
    }

    /// Creates [Program] instance to communicate with the selected program.
    pub fn program(&self, program_id: Pubkey) -> Program<Payer> {
        self.anchor_client.program(program_id).unwrap()
    }
}

/// Utility trait for printing transaction results.
pub trait PrintableTransaction {
    /// Pretty print the transaction results, tagged with the given name for distinguishability.
    fn print_named(&self, name: &str);

    /// Pretty print the transaction results.
    fn print(&self) {
        self.print_named("");
    }
}

impl PrintableTransaction for EncodedConfirmedTransactionWithStatusMeta {
    fn print_named(&self, name: &str) {
        let tx = self.transaction.transaction.decode().unwrap();
        log::debug!("EXECUTE {} (slot {})", name, self.slot);
        match self.transaction.meta.clone() {
            Some(meta) => println_transaction(&tx, Some(&meta), "  ", None, None),
            _ => println_transaction(&tx, None, "  ", None, None),
        }
    }
}

// @TODO remove once `Clone` is implemented for `Keypair`
// https://docs.rs/solana-sdk/latest/solana_sdk/signer/keypair/struct.Keypair.html

/// The `TempClone` trait is used as a workaround
/// for making non-cloneable foreign types cloneable.
pub trait TempClone {
    fn clone(&self) -> Self;
}

impl TempClone for Keypair {
    fn clone(&self) -> Self {
        Self::from_bytes(&self.to_bytes()).unwrap()
    }
}

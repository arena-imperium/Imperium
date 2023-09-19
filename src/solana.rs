// The deprecated `create_associated_token_account` function is used because of different versions
// of some crates are required in this `client` crate and `anchor-spl` crate
use {
    anchor_client::{
        anchor_lang::{prelude::System, Id},
        Client as AnchorClient, Cluster, Program,
    },
    bevy::{
        log,
        prelude::{Commands, Component, Resource},
        tasks::{IoTaskPool, Task},
    },
    hologram::{self},
    solana_cli_output::display::println_transaction,
    solana_client::rpc_config::RpcTransactionConfig,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
    },
    solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding},
    spl_associated_token_account::get_associated_token_address,
    std::{env, fmt, str::FromStr, sync::Arc},
    switchboard_solana::anchor_spl::token::spl_token::native_mint,
};

#[derive(Component)]
pub struct SolanaTransactionTask {
    pub description: String,
    pub task: Task<Result<EncodedConfirmedTransactionWithStatusMeta, SolanaTransactionTaskError>>,
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

#[derive(Resource)]
pub struct HologramServer {
    pub solana_client: Arc<SolanaClient>,
    pub realm_name: String,
    pub admin_pubkey: Pubkey,
    // Custom Switchboard functions
    pub spaceship_seed_generation_function: Pubkey,
    pub arena_matchmaking_function: Pubkey,
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
            realm_name: "Holorealm".to_string(), // @HARDCODED
            admin_pubkey: payer.pubkey().clone(),
            spaceship_seed_generation_function: Pubkey::from_str(
                "CyxB4ZrDSL2jjgPs5nGP93UpfNPHN4X66Z26WhnaeEi5",
            )
            .unwrap(), // @HARDCODED
            arena_matchmaking_function: Pubkey::from_str(
                "HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog",
            )
            .unwrap(), // @HARDCODED
        }
    }
}

// DEVNET test spaceship seed generation function CyxB4ZrDSL2jjgPs5nGP93UpfNPHN4X66Z26WhnaeEi5
// DEVNET test arena matchmaking function HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog
// https://app.switchboard.xyz/build/function/CyxB4ZrDSL2jjgPs5nGP93UpfNPHN4X66Z26WhnaeEi5
// https://app.switchboard.xyz/build/function/HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog

impl HologramServer {
    pub fn default_initialize_realm(&self, commands: &mut Commands) {
        self.initialize_realm(
            commands,
            &self.realm_name,
            &self.spaceship_seed_generation_function,
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn default_create_user_account(&self, commands: &mut Commands) {
        self.create_user_account(
            commands,
            self.realm_name.clone(),
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn default_create_spaceship(&self, commands: &mut Commands) {
        self.create_spaceship(
            commands,
            &"Nebuchadnezzar".to_string(),
            &self.solana_client.payer.pubkey(),
        );
    }

    pub fn initialize_realm(
        &self,
        commands: &mut Commands,
        realm_name: &String,
        randomness_function: &Pubkey,
        admin: &Pubkey, // Here should be a keypair, but it's just the payer. This IX is not really meant to be in this bevy app, just temporary for dev
    ) {
        log::info!("<Solana> Sending initialize_realm IX");
        let (realm_pda, _) = Self::get_realm_pda(&realm_name);
        let payer = self.solana_client.payer().clone();
        let admin = admin.clone();
        let realm_name = realm_name.clone();

        let program_id = hologram::id();
        let instruction = hologram::instruction::InitializeRealm { name: realm_name };
        let accounts = hologram::accounts::InitializeRealm {
            payer: payer.pubkey(),
            admin,
            realm: realm_pda,
            switchboard_function: randomness_function.clone(),
            system_program: solana_program::system_program::id(),
        };

        let task = self.create_send_and_confirm_instruction_task(
            program_id,
            instruction,
            accounts,
            payer.clone(),
            vec![],
            200_000,
        );

        commands.spawn(SolanaTransactionTask {
            description: "initialize_realm".to_string(),
            task,
        });
    }

    pub fn create_user_account(&self, commands: &mut Commands, realm_name: String, user: &Pubkey) {
        log::info!("<Solana> Sending create_user_account IX");
        let (realm_pda, _) = Self::get_realm_pda(&realm_name);
        let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, user);
        let payer = self.solana_client.payer().clone();
        let user = user.clone();

        let program_id = hologram::id();
        let instruction = hologram::instruction::CreateUserAccount {};
        let accounts = hologram::accounts::CreateUserAccount {
            user,
            realm: realm_pda,
            user_account: user_account_pda,
            system_program: solana_program::system_program::id(),
        };

        let task = self.create_send_and_confirm_instruction_task(
            program_id,
            instruction,
            accounts,
            payer.clone(),
            vec![],
            200_000,
        );

        commands.spawn(SolanaTransactionTask {
            description: "create_user_account".to_string(),
            task,
        });
    }

    pub fn create_spaceship(
        &self,
        commands: &mut Commands,
        spaceship_name: &String,
        user: &Pubkey,
    ) {
        log::info!("<Solana> Sending create_spaceship IX");

        // @HARDCODED: need to retrieve the user_account, and read the lenght of the spaceship vec.
        // but that require somt async code
        let spaceship_index = 0;

        let (realm_pda, _) = Self::get_realm_pda(&self.realm_name);
        let (user_account_pda, _) = Self::get_user_account_pda(&realm_pda, user);
        let (spaceship_pda, _) = Self::get_spaceship_pda(&realm_pda, &user, spaceship_index);
        let payer = self.solana_client.payer().clone();
        let user = user.clone();
        let spaceship_name = spaceship_name.clone();
        let user_wsol_token_account = get_associated_token_address(&user, &native_mint::ID);

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
        let program_id = hologram::id();
        let instruction = hologram::instruction::CreateSpaceship {
            name: spaceship_name,
        };

        let accounts = hologram::accounts::CreateSpaceship {
            user,
            realm: realm_pda,
            admin: self.admin_pubkey,
            user_account: user_account_pda,
            spaceship: spaceship_pda,
            switchboard_state: switchboard_state_pda,
            switchboard_attestation_queue: Pubkey::from_str(
                "CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE",
            )
            .unwrap(),
            spaceship_seed_generation_function: self.spaceship_seed_generation_function,
            switchboard_ssgf_request: switchboard_ssgf_request_keypair.pubkey(),
            switchboard_ssgf_request_escrow,
            arena_matchmaking_function: self.arena_matchmaking_function,
            switchboard_amf_request: switchboard_amf_request_keypair.pubkey(),
            switchboard_amf_request_escrow,
            user_wsol_token_account,
            switchboard_mint: native_mint::ID,
            system_program: solana_program::system_program::id(),
            token_program: switchboard_solana::anchor_spl::token::ID,
            switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
            associated_token_program: switchboard_solana::anchor_spl::associated_token::ID,
        };

        let task = self.create_send_and_confirm_instruction_task(
            program_id,
            instruction,
            accounts,
            payer.clone(),
            vec![
                switchboard_ssgf_request_keypair,
                switchboard_amf_request_keypair,
            ],
            250_000,
        );

        commands.spawn(SolanaTransactionTask {
            description: "create_spaceship".to_string(),
            task,
        });
    }

    /// Creates a task that sends and confirms an instruction to the Solana cluster.
    /// The task will be spawned on the `IoTaskPool` and will be polled on the main thread.
    /// The task will be removed from the `IoTaskPool` once it's completed.
    fn create_send_and_confirm_instruction_task(
        &self,
        program_id: Pubkey,
        instruction: impl anchor_client::anchor_lang::InstructionData + Send + 'static,
        accounts: impl anchor_client::anchor_lang::ToAccountMetas + Send + 'static,
        payer: Keypair,
        additionnal_signers: Vec<Keypair>,
        compute_budget_limit: u32,
    ) -> Task<Result<EncodedConfirmedTransactionWithStatusMeta, SolanaTransactionTaskError>> {
        let thread_pool = IoTaskPool::get();
        let client = Arc::clone(&self.solana_client);

        thread_pool.spawn(async move {
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
        })
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

    pub fn get_spaceship_pda(
        realm_pda: &Pubkey,
        user: &Pubkey,
        spaceship_index: usize,
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

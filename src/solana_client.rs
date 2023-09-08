// The deprecated `create_associated_token_account` function is used because of different versions
// of some crates are required in this `client` crate and `anchor-spl` crate
#[allow(deprecated)]
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use {
    anchor_client::{
        anchor_lang::{prelude::System, AccountDeserialize, Id, InstructionData, ToAccountMetas},
        Client as AnchorClient, ClientError as Error, Cluster, Program,
    },
    bevy::log,
    borsh::BorshDeserialize,
    fehler::throws,
    serde::de::DeserializeOwned,
    solana_account_decoder::parse_token::UiTokenAmount,
    solana_cli_output::display::println_transaction,
    solana_client::rpc_config::RpcTransactionConfig,
    solana_sdk::{
        account::Account,
        bpf_loader,
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        loader_instruction,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
    solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding},
    std::rc::Rc,
};
type Payer = Rc<Keypair>;

/// `Client` allows you to send typed RPC requests to a Solana cluster.
pub struct SolanaClient {
    payer: Keypair,
    anchor_client: AnchorClient<Payer>,
}
#[allow(dead_code)]
impl SolanaClient {
    /// Creates a new `Client` instance.
    pub fn new(payer: Keypair, cluster: Cluster) -> Self {
        Self {
            payer: payer.clone(),
            anchor_client: AnchorClient::new_with_options(
                cluster,
                Rc::new(payer),
                CommitmentConfig::confirmed(),
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

    // /// Finds out if the Solana localnet is running.
    // ///
    // /// Set `retry` to `true` when you want to wait for up to 15 seconds until
    // /// the localnet is running (until 30 retries with 500ms delays are performed).
    // pub async fn is_localnet_running(&self, retry: bool) -> bool {
    //     let rpc_client = self
    //         .anchor_client
    //         .program(System::id())
    //         .unwrap()
    //         .async_rpc();

    //     for _ in 0..(if retry {
    //         CONFIG.test.validator_startup_timeout / RETRY_LOCALNET_EVERY_MILLIS
    //     } else {
    //         1
    //     }) {
    //         if rpc_client.get_health().await.is_ok() {
    //             return true;
    //         }
    //         if retry {
    //             sleep(Duration::from_millis(RETRY_LOCALNET_EVERY_MILLIS));
    //         }
    //     }
    //     false
    // }

    /// Gets deserialized data from the chosen account serialized with Anchor
    ///
    /// # Errors
    ///
    /// It fails when:
    /// - the account does not exist.
    /// - the Solana cluster is not running.
    /// - deserialization failed.
    #[throws]
    pub async fn account_data<T>(&self, account: Pubkey) -> T
    where
        T: AccountDeserialize + Send + 'static,
    {
        let program = self.anchor_client.program(System::id()).unwrap();
        program.account::<T>(account).unwrap()
    }

    /// Gets deserialized data from the chosen account serialized with Bincode
    ///
    /// # Errors
    ///
    /// It fails when:
    /// - the account does not exist.
    /// - the Solana cluster is not running.
    /// - deserialization failed.
    #[throws]
    pub async fn account_data_bincode<T>(&self, account: Pubkey) -> T
    where
        T: DeserializeOwned + Send + 'static,
    {
        let account = self
            .get_account(account)
            .await?
            .ok_or(Error::AccountNotFound)?;

        bincode::deserialize(&account.data)
            .map_err(|_| Error::LogParseError("Bincode deserialization failed".to_string()))?
    }

    /// Gets deserialized data from the chosen account serialized with Borsh
    ///
    /// # Errors
    ///
    /// It fails when:
    /// - the account does not exist.
    /// - the Solana cluster is not running.
    /// - deserialization failed.
    #[throws]
    pub async fn account_data_borsh<T>(&self, account: Pubkey) -> T
    where
        T: BorshDeserialize + Send + 'static,
    {
        let account = self
            .get_account(account)
            .await?
            .ok_or(Error::AccountNotFound)?;

        T::try_from_slice(&account.data)
            .map_err(|_| Error::LogParseError("Bincode deserialization failed".to_string()))?
    }

    /// Returns all information associated with the account of the provided [Pubkey].
    ///
    /// # Errors
    ///
    /// It fails when the Solana cluster is not running.
    #[throws]
    pub async fn get_account(&self, account: Pubkey) -> Option<Account> {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        rpc_client
            .get_account_with_commitment(&account, rpc_client.commitment())
            .await
            .unwrap()
            .value
    }

    /// Sends the Anchor instruction with associated accounts and signers.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use trdelnik_client::*;
    ///
    /// pub async fn initialize(
    ///     client: &Client,
    ///     state: Pubkey,
    ///     user: Pubkey,
    ///     system_program: Pubkey,
    ///     signers: impl IntoIterator<Item = Keypair> + Send + 'static,
    /// ) -> Result<EncodedConfirmedTransactionWithStatusMeta, ClientError> {
    ///     Ok(client
    ///         .send_instruction(
    ///             PROGRAM_ID,
    ///             turnstile::instruction::Initialize {},
    ///             turnstile::accounts::Initialize {
    ///                 state: a_state,
    ///                 user: a_user,
    ///                 system_program: a_system_program,
    ///             },
    ///             signers,
    ///         )
    ///         .await?)
    /// }
    /// ```
    #[throws]
    pub async fn send_instruction(
        &self,
        program: Pubkey,
        instruction: impl InstructionData + Send + 'static,
        accounts: impl ToAccountMetas + Send + 'static,
        signers: impl IntoIterator<Item = Keypair> + Send + 'static,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let program = self.anchor_client.program(program).unwrap();
        let mut request = program.request().args(instruction).accounts(accounts);
        let signers = signers.into_iter().collect::<Vec<_>>();
        for signer in &signers {
            request = request.signer(signer);
        }
        let signature = request.send().unwrap();

        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Binary),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: None,
                },
            )
            .await
            .unwrap()
    }

    /// Sends the transaction with associated instructions and signers.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[throws]
    /// pub async fn create_account(
    ///     &self,
    ///     keypair: &Keypair,
    ///     lamports: u64,
    ///     space: u64,
    ///     owner: &Pubkey,
    /// ) -> EncodedConfirmedTransaction {
    ///     self.send_transaction(
    ///         &[system_instruction::create_account(
    ///             &self.payer().pubkey(),
    ///             &keypair.pubkey(),
    ///             lamports,
    ///             space,
    ///             owner,
    ///         )],
    ///         [keypair],
    ///     )
    ///     .await?
    /// }
    /// ```
    #[throws]
    pub async fn send_transaction(
        &self,
        instructions: &[Instruction],
        signers: impl IntoIterator<Item = &Keypair> + Send,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let rpc_client = self
            .anchor_client
            .program(System::id())
            .unwrap()
            .async_rpc();
        let mut signers = signers.into_iter().collect::<Vec<_>>();
        signers.push(self.payer());

        let tx = &Transaction::new_signed_with_payer(
            instructions,
            Some(&self.payer.pubkey()),
            &signers,
            rpc_client.get_latest_blockhash().await.unwrap(),
        );
        // @TODO make this call async with task::spawn_blocking
        let signature = rpc_client.send_and_confirm_transaction(tx).await.unwrap();
        let transaction = rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Binary),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: None,
                },
            )
            .await
            .unwrap();

        transaction
    }

    // /// Airdrops lamports to the chosen account.
    // #[throws]
    // pub async fn airdrop(&self, address: Pubkey, lamports: u64) {
    //     let rpc_client = self
    //         .anchor_client
    //         .program(System::id())
    //         .unwrap()
    //         .async_rpc();

    //     let signature = rpc_client
    //         .request_airdrop(&address, lamports)
    //         .await
    //         .unwrap();

    //     let (airdrop_result, error) = loop {
    //         match rpc_client.get_signature_status(&signature).await.unwrap() {
    //             Some(Ok(_)) => {
    //                 debug!("{} lamports airdropped", lamports);
    //                 break (true, None);
    //             }
    //             Some(Err(transaction_error)) => break (false, Some(transaction_error)),
    //             None => sleep(Duration::from_millis(500)),
    //         }
    //     };
    //     if !airdrop_result {
    //         throw!(Error::SolanaClientError(error.unwrap().into()));
    //     }
    // }

    /// Get balance of an account
    #[throws]
    pub async fn get_balance(&mut self, address: &Pubkey) -> u64 {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        rpc_client.get_balance(address).await?
    }

    /// Get token balance of an token account
    #[throws]
    pub async fn get_token_balance(&mut self, address: Pubkey) -> UiTokenAmount {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        rpc_client.get_token_account_balance(&address).await?
    }

    /// Creates accounts.
    #[throws]
    pub async fn create_account(
        &self,
        keypair: &Keypair,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        self.send_transaction(
            &[system_instruction::create_account(
                &self.payer().pubkey(),
                &keypair.pubkey(),
                lamports,
                space,
                owner,
            )],
            [keypair],
        )
        .await?
    }

    /// Creates rent exempt account.
    #[throws]
    pub async fn create_account_rent_exempt(
        &mut self,
        keypair: &Keypair,
        space: u64,
        owner: &Pubkey,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        self.send_transaction(
            &[system_instruction::create_account(
                &self.payer().pubkey(),
                &keypair.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(space as usize)
                    .await?,
                space,
                owner,
            )],
            [keypair],
        )
        .await?
    }

    /// Executes a transaction constructing a token mint.
    #[throws]
    pub async fn create_token_mint(
        &self,
        mint: &Keypair,
        authority: Pubkey,
        freeze_authority: Option<Pubkey>,
        decimals: u8,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        self.send_transaction(
            &[
                system_instruction::create_account(
                    &self.payer().pubkey(),
                    &mint.pubkey(),
                    rpc_client
                        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
                        .await?,
                    spl_token::state::Mint::LEN as u64,
                    &spl_token::ID,
                ),
                spl_token::instruction::initialize_mint(
                    &spl_token::ID,
                    &mint.pubkey(),
                    &authority,
                    freeze_authority.as_ref(),
                    decimals,
                )
                .unwrap(),
            ],
            [mint],
        )
        .await?
    }

    /// Executes a transaction that mints tokens from a mint to an account belonging to that mint.
    #[throws]
    pub async fn mint_tokens(
        &self,
        mint: Pubkey,
        authority: &Keypair,
        account: Pubkey,
        amount: u64,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        self.send_transaction(
            &[spl_token::instruction::mint_to(
                &spl_token::ID,
                &mint,
                &account,
                &authority.pubkey(),
                &[],
                amount,
            )
            .unwrap()],
            [authority],
        )
        .await?
    }

    /// Executes a transaction constructing a token account of the specified mint. The account needs to be empty and belong to system for this to work.
    /// Prefer to use [create_associated_token_account] if you don't need the provided account to contain the token account.
    #[throws]
    pub async fn create_token_account(
        &self,
        account: &Keypair,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> EncodedConfirmedTransactionWithStatusMeta {
        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        self.send_transaction(
            &[
                system_instruction::create_account(
                    &self.payer().pubkey(),
                    &account.pubkey(),
                    rpc_client
                        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
                        .await?,
                    spl_token::state::Account::LEN as u64,
                    &spl_token::ID,
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::ID,
                    &account.pubkey(),
                    mint,
                    owner,
                )
                .unwrap(),
            ],
            [account],
        )
        .await?
    }

    /// Executes a transaction constructing the associated token account of the specified mint belonging to the owner. This will fail if the account already exists.
    #[throws]
    pub async fn create_associated_token_account(&self, owner: &Keypair, mint: Pubkey) -> Pubkey {
        self.send_transaction(
            #[allow(deprecated)]
            &[create_associated_token_account(
                &self.payer().pubkey(),
                &owner.pubkey(),
                &mint,
            )],
            &[],
        )
        .await?;
        get_associated_token_address(&owner.pubkey(), &mint)
    }

    /// Executes a transaction creating and filling the given account with the given data.
    /// The account is required to be empty and will be owned by bpf_loader afterwards.
    #[throws]
    pub async fn create_account_with_data(&self, account: &Keypair, data: Vec<u8>) {
        const DATA_CHUNK_SIZE: usize = 900;

        let rpc_client = self.anchor_client.program(System::id())?.async_rpc();
        self.send_transaction(
            &[system_instruction::create_account(
                &self.payer().pubkey(),
                &account.pubkey(),
                rpc_client
                    .get_minimum_balance_for_rent_exemption(data.len())
                    .await?,
                data.len() as u64,
                &bpf_loader::id(),
            )],
            [account],
        )
        .await?;

        let mut offset = 0usize;
        for chunk in data.chunks(DATA_CHUNK_SIZE) {
            log::debug!("writing bytes {} to {}", offset, offset + chunk.len());
            self.send_transaction(
                &[loader_instruction::write(
                    &account.pubkey(),
                    &bpf_loader::id(),
                    offset as u32,
                    chunk.to_vec(),
                )],
                [account],
            )
            .await?;
            offset += chunk.len();
        }
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

pub fn get_realm_pda(realm_name: &String) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"realm", realm_name.as_bytes()], &hologram::id())
}

pub fn get_user_account_pda(realm_pda: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_account", realm_pda.as_ref(), user.as_ref()],
        &hologram::id(),
    )
}

pub fn get_map_pda(realm_pda: &Pubkey, user_account_pda: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"map", realm_pda.as_ref(), user_account_pda.as_ref()],
        &hologram::id(),
    )
}

pub fn get_map_entities_pda(
    realm_pda: &Pubkey,
    user_account_pda: &Pubkey,
    map_pda: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"map_entities",
            realm_pda.as_ref(),
            user_account_pda.as_ref(),
            map_pda.as_ref(),
        ],
        &hologram::id(),
    )
}

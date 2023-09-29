use {
    crate::utils::pda,
    anchor_client::ClientError,
    anchor_lang::{prelude::*, InstructionData},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_program::{
        bpf_loader_upgradeable, clock::DEFAULT_MS_PER_SLOT, epoch_schedule::DEFAULT_SLOTS_PER_EPOCH,
    },
    solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext},
    solana_sdk::{
        account::Account, commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction, pubkey::Pubkey, signers::Signers,
    },
    tokio::sync::RwLock,
};

// Copy accounts from *net to localnet
pub async fn clone_accounts_to_localnet(
    program_test: &mut ProgramTest,
    accounts_keys: &[Pubkey],
    origin: String,
) -> std::result::Result<(), ClientError> {
    {
        let devnet_client = RpcClient::new(origin);

        let accounts = devnet_client
            .get_multiple_accounts_with_commitment(&accounts_keys, CommitmentConfig::default())
            .await?
            .value;

        // Check if there are program accounts
        for (account, acc_key) in accounts.iter().zip(accounts_keys) {
            if let Some(account) = account {
                // add the account
                program_test.add_account(*acc_key, account.clone());
                // additionnally: if it's a program, fetch and add the programData account
                if account.owner == bpf_loader_upgradeable::id() {
                    let (program_data_pda, _) = pda::get_program_data_pda(acc_key);

                    let program_data_account = devnet_client
                        .get_account_with_commitment(&program_data_pda, CommitmentConfig::default())
                        .await?
                        .value
                        .unwrap();

                    program_test.add_account(program_data_pda, program_data_account);
                }
            } else {
                return Err(ClientError::AccountNotFound);
            }
        }
        Ok(())
    }
}

pub fn create_and_fund_account(address: &Pubkey, program_test: &mut ProgramTest) {
    program_test.add_account(
        *address,
        Account {
            lamports: 1_000_000_000, // 1 sol
            ..Account::default()
        },
    );
}

pub async fn create_and_execute_hologram_ix<T: InstructionData, U: Signers>(
    program_test_ctx: &RwLock<ProgramTestContext>,
    accounts_meta: Vec<AccountMeta>,
    args: T,
    payer: Option<&Pubkey>,
    signing_keypairs: &U,
    pre_ix: Option<solana_sdk::instruction::Instruction>,
    post_ix: Option<solana_sdk::instruction::Instruction>,
) -> std::result::Result<(), BanksClientError> {
    let ix = solana_sdk::instruction::Instruction {
        program_id: hologram::id(),
        accounts: accounts_meta,
        data: args.data(),
    };
    let mut ctx = program_test_ctx.write().await;
    let last_blockhash = ctx.last_blockhash;
    let banks_client = &mut ctx.banks_client;
    let mut instructions: Vec<solana_sdk::instruction::Instruction> = Vec::new();
    instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(400_000u32));
    if pre_ix.is_some() {
        instructions.push(pre_ix.unwrap());
    }
    instructions.push(ix);
    if post_ix.is_some() {
        instructions.push(post_ix.unwrap());
    }
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        instructions.as_slice(),
        payer,
        signing_keypairs,
        last_blockhash,
    );
    let result = banks_client.process_transaction(tx).await;
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    Ok(())
}

pub async fn get_account<T: anchor_lang::AccountDeserialize>(
    program_test_ctx: &RwLock<ProgramTestContext>,
    key: &Pubkey,
) -> T {
    let mut ctx = program_test_ctx.write().await;
    let banks_client = &mut ctx.banks_client;

    let account = banks_client.get_account(*key).await.unwrap().unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}

// Doesn't check if you go before epoch 0 when passing negative amounts, be wary
pub async fn warp_forward(ctx: &RwLock<ProgramTestContext>, seconds: i64) {
    let mut ctx = ctx.write().await;

    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "Original Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );
    let mut new_clock = clock_sysvar.clone();
    new_clock.unix_timestamp += seconds;

    let seconds_since_epoch_start = new_clock.unix_timestamp - clock_sysvar.epoch_start_timestamp;
    let ms_since_epoch_start = seconds_since_epoch_start * 1_000;
    let slots_since_epoch_start = ms_since_epoch_start / DEFAULT_MS_PER_SLOT as i64;
    let epochs_since_epoch_start = slots_since_epoch_start / DEFAULT_SLOTS_PER_EPOCH as i64;
    new_clock.epoch = (new_clock.epoch as i64 + epochs_since_epoch_start) as u64;

    ctx.set_sysvar(&new_clock);
    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "New Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );

    let blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    ctx.last_blockhash = blockhash;
}

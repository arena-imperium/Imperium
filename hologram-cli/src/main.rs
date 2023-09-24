pub mod error;

use {
    crate::error::CliError,
    anchor_client::{
        solana_sdk::{signature::Signer, signer::keypair},
        Cluster,
    },
    clap::{Parser, Subcommand},
    solana_program::pubkey::Pubkey,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(about = "Hologram program CLI", long_about = None)]
struct Cli {
    /// RPC endpoint.
    #[clap(long, env = "SOLANA_RPC_URL")]
    rpc_url: String,

    /// Websocket endpoint.
    #[clap(long, env = "SOLANA_WS_URL")]
    ws_url: String,

    /// Path to keypair
    #[clap(short, long, env = "SOLANA_PAYER_KEY")]
    payer_keypair_path: std::path::PathBuf,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    InitializeRealm {
        #[clap(long, env = "SWITCHBOARD_SSGF")]
        switchboard_ssgf: Pubkey,

        #[clap(long, env = "SWITCHBOARD_AMF")]
        switchboard_amf: Pubkey,

        #[clap(short, long, default_value = "HoloRealm")]
        realm_name: String,
    },
}

fn main() -> Result<(), CliError> {
    dotenv::dotenv().ok();
    let Cli {
        rpc_url,
        ws_url,
        payer_keypair_path,
        command,
    } = Cli::parse();

    let cluster = Cluster::Custom(rpc_url, ws_url);

    let payer = keypair::read_keypair_file(&payer_keypair_path).unwrap();

    let client = anchor_client::Client::new(cluster, &payer);
    let hologram_program = client.program(hologram::id()).unwrap();

    match command {
        Command::InitializeRealm {
            realm_name,
            switchboard_ssgf,
            switchboard_amf,
        } => {
            let (realm_pda, _) = get_realm_pda(&realm_name);
            let tx = hologram_program
                .request()
                .accounts(hologram::accounts::InitializeRealm {
                    payer: payer.pubkey(),
                    admin: payer.pubkey(),
                    realm: realm_pda,
                    spaceship_seed_generation_function: switchboard_ssgf,
                    arena_matchmaking_function: switchboard_amf,
                    system_program: solana_program::system_program::id(),
                })
                .args(hologram::instruction::InitializeRealm { name: realm_name })
                .signer(&payer)
                .send()?;

            println!("tx: {}", tx);
        }
    };

    Ok(())
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

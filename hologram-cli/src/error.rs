#[derive(thiserror::Error, Debug)]
pub enum CliError {
    // Library errors
    #[error("{0}: {0:?}")]
    AnchorClient(#[from] anchor_client::ClientError),
    #[error("{0}: {0:?}")]
    SolanaClient(#[from] solana_client::client_error::ClientError),
    #[error("{0}: {0:?}")]
    TransactionError(#[from] anchor_client::solana_sdk::transaction::TransactionError),
    #[error("{0}")]
    Var(#[from] std::env::VarError),
}

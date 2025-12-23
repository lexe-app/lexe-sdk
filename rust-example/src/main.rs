//! Example usage of the Lexe Rust SDK.
//!
//! See the README.md for setup instructions.

use std::{env, path::PathBuf, str::FromStr};

use lexe_sdk::{
    anyhow::{self, anyhow, Context},
    config::WalletEnvConfig,
    tracing::info,
    types::{
        ClientCredentials, Credentials, RootSeed, SdkCreateInvoiceRequest,
        SysRng,
    },
    wallet::LexeWallet,
};

fn main() -> anyhow::Result<()> {
    // (Optional) Load env vars from .env.
    let _ = dotenvy::dotenv();

    // (Optional) Set up Lexe's `tracing` logger.
    lexe_sdk::init_logger("info");
    info!("Initializing program.");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to build Tokio runtime")?;

    rt.block_on(run())
}

async fn run() -> anyhow::Result<()> {
    // Parse credentials from environment.
    // Accepts `ROOT_SEED` (hex-encoded 32 bytes) or
    // `LEXE_CLIENT_CREDENTIALS` (base64 JSON blob).
    let credentials = if let Ok(root_seed_hex) = env::var("ROOT_SEED") {
        let root_seed = RootSeed::from_str(&root_seed_hex)
            .context("Failed to parse ROOT_SEED (expected 64 hex chars)")?;
        info!("Using ROOT_SEED credentials");
        Credentials::RootSeed(root_seed)
    } else if let Ok(credentials_str) = env::var("LEXE_CLIENT_CREDENTIALS") {
        let credentials =
            ClientCredentials::try_from_base64_blob(&credentials_str)
                .context("Failed to parse LEXE_CLIENT_CREDENTIALS")?;
        info!("Using LEXE_CLIENT_CREDENTIALS");
        Credentials::ClientCredentials(credentials)
    } else {
        return Err(anyhow!(
            "No credentials found. Set either ROOT_SEED (hex) or \
                 LEXE_CLIENT_CREDENTIALS (base64) in env or .env"
        ));
    };

    // Data directory for persistence. Defaults to `.lexe_data` in the current
    // directory. Override with LEXE_DATA_DIR environment variable.
    let lexe_data_dir = env::var("LEXE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".lexe_data"));

    // Initialize cryptographic RNG.
    let mut rng = SysRng::new();

    // Create a wallet config for mainnet.
    let env_config = WalletEnvConfig::prod();

    // Load or create wallet
    let wallet = LexeWallet::load_or_fresh(
        &mut rng,
        env_config,
        credentials.as_ref(),
        lexe_data_dir,
    )
    .context("Failed to load wallet")?;

    // Ensure the node is provisioned to the latest versions
    let allow_gvfs_access = false;
    let encrypted_seed = None;
    let google_auth_code = None;
    wallet
        .ensure_provisioned(
            credentials.as_ref(),
            allow_gvfs_access,
            encrypted_seed,
            google_auth_code,
        )
        .await
        .context("Failed to ensure node is provisioned")?;
    info!("Node is provisioned and ready");

    // Get node info
    let node_info = wallet
        .node_info()
        .await
        .context("Failed to get node info")?;
    let node_info_json = lexe_sdk::serde_json::to_string_pretty(&node_info)
        .context("Failed to serialize node info")?;
    info!("Node info:\n{node_info_json}");

    // Sync payments from the node
    let sync_summary = wallet
        .sync_payments()
        .await
        .context("Failed to sync payments")?;
    info!(
        new = sync_summary.num_new,
        updated = sync_summary.num_updated,
        "Payment sync complete"
    );

    // Create an invoice
    let create_invoice_req = SdkCreateInvoiceRequest {
        expiration_secs: 3600, // 1 hour
        amount: None,          // Amountless invoice
        description: Some("Test invoice from rust-example".to_string()),
    };
    let invoice_resp = wallet
        .create_invoice(create_invoice_req)
        .await
        .context("Failed to create invoice")?;
    info!(invoice = %invoice_resp.invoice, "Created invoice");

    // Get the latest synced payment from local db
    let payment = wallet
        .payments_db()
        .get_payment_by_scroll_idx(0)
        .context("Expected at least one payment after sync")?;
    let payment_json = lexe_sdk::serde_json::to_string_pretty(&payment)
        .context("Failed to serialize payment")?;
    info!("Latest synced payment:\n{payment_json}");

    Ok(())
}

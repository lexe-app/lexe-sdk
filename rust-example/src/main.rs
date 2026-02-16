//! Example usage of the Lexe Rust SDK.
//!
//! Run this example:
//!
//! ```bash
//! $ cd rust-example/
//! $ cargo run
//! ```
//!
//! See the README.md for setup instructions.

use std::{env, path::PathBuf, str::FromStr};

use lexe_sdk::{
    anyhow::{self, Context},
    config::WalletEnvConfig,
    tracing::info,
    types::{
        ClientCredentials, Credentials, RootSeed, SdkCreateInvoiceRequest,
        SdkGetPaymentRequest, SysRng,
    },
    wallet::{LexeWallet, default_lexe_data_dir},
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

#[tracing::instrument(name = "rust-example")]
async fn run() -> anyhow::Result<()> {
    // Initialize cryptographic RNG.
    let mut rng = SysRng::new();

    // Create a wallet config for testnet3.
    // For Bitcoin mainnet, use `WalletEnvConfig::mainnet()`.
    let env_config = WalletEnvConfig::testnet3();

    // Data directory for persistence. Defaults to `~/.lexe`.
    let lexe_data_dir = env::var("LEXE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_lexe_data_dir()?);

    // Various ways of loading or creating Lexe credentials:
    // 1. LEXE_CLIENT_CREDENTIALS env var (base64 JSON blob)
    // 2. LEXE_ROOT_SEED env var (hex string)
    // 3. Read seedphrase file from data dir
    // 4. Generate fresh RootSeed and write to data dir
    let seedphrase_path = env_config.seedphrase_path(&lexe_data_dir);
    let credentials = if let Ok(creds_str) = env::var("LEXE_CLIENT_CREDENTIALS")
    {
        let creds = ClientCredentials::try_from_base64_blob(&creds_str)
            .context("Failed to parse LEXE_CLIENT_CREDENTIALS")?;
        info!("Using LEXE_CLIENT_CREDENTIALS");
        Credentials::ClientCredentials(creds)
    } else if let Ok(seed_hex) = env::var("LEXE_ROOT_SEED") {
        let root_seed = RootSeed::from_str(&seed_hex)
            .context("Failed to parse LEXE_ROOT_SEED (expected 64 hex)")?;
        info!("Using LEXE_ROOT_SEED");
        Credentials::RootSeed(root_seed)
    } else if let Some(root_seed) =
        RootSeed::read_from_seedphrase_file(&seedphrase_path)?
    {
        info!(path = %seedphrase_path.display(), "Loaded seedphrase");
        Credentials::RootSeed(root_seed)
    } else {
        info!("No credentials found, generating fresh RootSeed");
        let root_seed = RootSeed::from_rng(&mut rng);
        root_seed.write_to_seedphrase_file(&seedphrase_path)?;
        info!(path = %seedphrase_path.display(), "Wrote seedphrase");
        Credentials::RootSeed(root_seed)
    };

    // Load or create wallet (uses default data dir ~/.lexe)
    let wallet = LexeWallet::load_or_fresh(
        &mut rng,
        env_config,
        credentials.as_ref(),
        None,
    )
    .context("Failed to load wallet")?;

    // Initial signup and provisioning of the node.
    // This operation is idempotent, so it's ok to call if already done.
    if let Credentials::RootSeed(ref root_seed) = credentials {
        // Set to your company's UserPk to earn a share of this wallet's fees.
        let partner_pk = None;
        wallet
            .signup(&mut rng, root_seed, partner_pk)
            .await
            .context("Failed to signup and provision")?;
        info!("Signup and provision complete");
    }

    // For existing wallets, always call provision to ensure we're up-to-date.
    // This operation is also idempotent.
    wallet
        .provision(credentials.as_ref())
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

    // Get the payment we just created
    let get_payment_req = SdkGetPaymentRequest {
        index: invoice_resp.index,
    };
    let payment = wallet
        .get_payment(get_payment_req)
        .await
        .context("Failed to get payment")?
        .payment
        .context("Payment not found")?;
    let payment_json = lexe_sdk::serde_json::to_string_pretty(&payment)
        .context("Failed to serialize payment")?;
    info!("Created payment:\n{payment_json}");

    Ok(())
}

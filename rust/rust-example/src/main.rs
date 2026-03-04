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

use std::{env, str::FromStr};

use lexe::{
    anyhow::{self, Context},
    config::WalletEnvConfig,
    tracing::info,
    types::{
        auth::{ClientCredentials, Credentials, RootSeed},
        command::{SdkCreateInvoiceRequest, SdkGetPaymentRequest},
    },
    wallet::LexeWallet,
};

fn main() -> anyhow::Result<()> {
    // (Optional) Load env vars from .env.
    let _ = dotenvy::dotenv();

    // (Optional) Set up Lexe's `tracing` logger.
    lexe::init_logger("info");
    info!("Initializing program.");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to build Tokio runtime")?;

    rt.block_on(run())
}

#[tracing::instrument(name = "rust-example")]
async fn run() -> anyhow::Result<()> {
    // Create a wallet config for testnet3.
    // For Bitcoin mainnet, use `WalletEnvConfig::mainnet()`.
    let env_config = WalletEnvConfig::testnet3();

    // Various ways of loading or creating Lexe credentials:
    // 1. LEXE_CLIENT_CREDENTIALS env var (base64 JSON blob)
    // 2. LEXE_ROOT_SEED env var (hex string)
    // 3. Read seedphrase file from ~/.lexe
    // 4. Generate fresh RootSeed and write to ~/.lexe
    let lexe_data_dir = lexe::default_lexe_data_dir()?;
    let seedphrase_path = env_config.seedphrase_path(&lexe_data_dir);
    let is_new_seed;
    let credentials = if let Ok(creds_str) = env::var("LEXE_CLIENT_CREDENTIALS")
    {
        let creds = ClientCredentials::try_from_base64_blob(&creds_str)
            .context("Failed to parse LEXE_CLIENT_CREDENTIALS")?;
        info!("Using LEXE_CLIENT_CREDENTIALS");
        is_new_seed = false;
        Credentials::ClientCredentials(creds)
    } else if let Ok(seed_hex) = env::var("LEXE_ROOT_SEED") {
        let root_seed = RootSeed::from_str(&seed_hex)
            .context("Failed to parse LEXE_ROOT_SEED (expected 64 hex)")?;
        info!("Using LEXE_ROOT_SEED");
        is_new_seed = false;
        Credentials::RootSeed(root_seed)
    } else if let Some(root_seed) = env_config.read_seed()? {
        info!("Loaded seedphrase from {}", seedphrase_path.display());
        is_new_seed = false;
        Credentials::RootSeed(root_seed)
    } else {
        info!("No credentials found, generating fresh RootSeed");
        is_new_seed = true;
        Credentials::RootSeed(RootSeed::generate())
    };

    // Load or create wallet (data stored in ~/.lexe)
    let lexe_data_dir = None; // Use ~/.lexe by default, set to override
    let wallet = LexeWallet::load_or_fresh(
        env_config.clone(),
        credentials.as_ref(),
        lexe_data_dir,
    )
    .context("Failed to load wallet")?;

    if let Credentials::RootSeed(ref root_seed) = credentials {
        if is_new_seed {
            // Initial signup and provisioning of the node.
            // This is idempotent: calling this for existing users is safe.
            // Set partner_pk to your company's UserPk to earn fee revenue.
            let partner_pk = None;
            wallet
                .signup(root_seed, partner_pk)
                .await
                .context("Failed to signup and provision")?;
            info!("Signup and initial provision complete");

            // Persist the seed. The next time we run, we'll read the existing seed.
            env_config.write_seed(root_seed)?;
            info!("Wrote seedphrase to {}", seedphrase_path.display());
        } else {
            // Ensure we're provisioned to all recent trusted releases.
            // Trusted releases are in `releases.json` in the Lexe repository.
            //
            // TODO(max): Can also allow ClientCredentials to provision once
            // delegated provisioning is implemented.
            wallet
                .provision(credentials.as_ref())
                .await
                .context("Failed to ensure node is provisioned")?;
            info!("Node is provisioned to latest release");
        }
    }

    // Get node info
    let node_info = wallet
        .node_info()
        .await
        .context("Failed to get node info")?;
    let node_info_json = lexe::serde_json::to_string_pretty(&node_info)
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
    let payment_json = lexe::serde_json::to_string_pretty(&payment)
        .context("Failed to serialize payment")?;
    info!("Created payment:\n{payment_json}");

    Ok(())
}

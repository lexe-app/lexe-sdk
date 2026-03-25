# Lexe Rust SDK

The Lexe Rust SDK provides a native Rust interface for developers to control their
self-custodial, always-online [Lexe](https://lexe.app) Lightning node.

This repo contains public-facing docs and examples for the Lexe Rust SDK.
The `lexe` crate source code can be found in the Lexe monorepo at:
<https://github.com/lexe-app/lexe-public/tree/master/lexe>

# Quickstart

### Download the Lexe Wallet app

To get started, you'll need a Lexe wallet. Download the Lexe mobile app for:

* iOS (Testflight): <https://lexe.app/beta-ios>
* Android (Play Store): <https://lexe.app/beta-android>

> **Note for iOS users**: TestFlight sometimes asks for an "access code".
> If you see this prompt, try clicking the download link again.
> If that doesn't work, reinstall the TestFlight app and retry.

After installing the app, select "Create a new wallet", optionally connect your
Google Drive (for backups), and set a backup password. This will create a new
self-custody Bitcoin+Lightning wallet that runs in a secure enclave in the
cloud; you can now send and receive Lightning payments 24/7!

### Configure credentials for the SDK

The Rust SDK supports two credential types:

- `ClientCredentials`: control an existing wallet created in the Lexe app.
- `RootSeed`: sign up users and manage wallets from your backend.

**Option 1: `ClientCredentials` (existing app wallet)**

To control a wallet created in the app, export client credentials:

1. Open the Lexe app > Menu sidebar > "SDK clients" > "Create new client"
2. Copy the client credentials string

Set `LEXE_CLIENT_CREDENTIALS` in your environment or in a `.env` file:

```bash
# Option 1: Set directly in environment
$ export LEXE_CLIENT_CREDENTIALS="eyJsZXhlX2F1dGhfdG9rZ...TA0In0"

# Option 2: Use a .env file which the example loads automatically
$ cp .env.example .env
$ chmod 600 .env
# Then edit .env and set LEXE_CLIENT_CREDENTIALS
```

**Option 2: `RootSeed` (programmatic signup)**

To sign up users and manage wallets from your backend, use `RootSeed`
credentials.
In the example below, the caller generates a fresh root seed on first run and
stores it at `~/.lexe/seedphrase.txt`.

### Using the SDK

See the [rust-example](rust-example) directory for a complete example showing
how to:

- Initialize a `LexeWallet` with credentials
- Ensure the node is provisioned to the latest version
- Query node info and balance
- Create Lightning invoices
- Sync and query payments from the local database

Add `lexe` to your `Cargo.toml`:

```toml
[dependencies]
lexe = "0.1"
```

Basic usage:

```rust
use std::str::FromStr;

use lexe::{
    config::WalletEnvConfig,
    types::{
        auth::{CredentialsRef, RootSeed},
        bitcoin::Invoice,
        command::{CreateInvoiceRequest, PayInvoiceRequest},
    },
    wallet::LexeWallet,
};

// Create a wallet config for mainnet (or testnet3() for testing)
let env_config = WalletEnvConfig::mainnet();

// Load root seed from ~/.lexe, or create a fresh one
let is_new_seed;
let root_seed = match RootSeed::read(&env_config.wallet_env)? {
    Some(seed) => {
        is_new_seed = false;
        seed
    }
    None => {
        is_new_seed = true;
        RootSeed::generate()
    }
};
let credentials = CredentialsRef::from(&root_seed);

// Load or create wallet (data stored in ~/.lexe)
let wallet =
    LexeWallet::load_or_fresh(env_config.clone(), credentials, None)?;

if is_new_seed {
    // Signup with Lexe and provision the node (idempotent)
    let partner_pk = None;
    wallet.signup(&root_seed, partner_pk).await?;
    root_seed.write(&env_config.wallet_env)?;
} else {
    // Ensure provisioned to latest trusted release
    wallet.provision(credentials).await?;
}

// Get node info
let node_info = wallet.node_info().await?;
println!("Balance: {} sats", node_info.balance);

// Create a Lightning invoice
let invoice_req = CreateInvoiceRequest {
    expiration_secs: 3600,
    amount: None,
    description: Some("VPN subscription (1 month)".to_string()),
    payer_note: None,
};
let invoice_resp = wallet.create_invoice(invoice_req).await?;

// Pay an invoice
let invoice = Invoice::from_str("lnbc1pjlue...")?;
let pay_req = PayInvoiceRequest {
    invoice,
    fallback_amount: None,
    note: Some("Mass-produced mass-market Miller Lite".to_string()),
    payer_note: None,
};
let pay_resp = wallet.pay_invoice(pay_req).await?;

// Sync payments from the node
let summary = wallet.sync_payments().await?;
println!("Synced {} new payments", summary.num_new);
```

Run the example:

```bash
cd rust-example
cargo run
```

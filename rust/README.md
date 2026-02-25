# Lexe Rust SDK

The Lexe Rust SDK provides a native Rust interface for developers to control their
self-custodial, always-online [Lexe](https://lexe.app) Lightning node.

This repo contains public-facing docs and examples for the Lexe Rust SDK.
The `lexe` crate source code can be found in the Lexe monorepo at:
<https://github.com/lexe-app/lexe-public/tree/master/lexe>

# Quickstart

### Download the Lexe Wallet app

To get started, you'll need a Lexe wallet. Developers building on the Rust SDK
are eligible for early access to Lexe wallet, which is currently in closed beta.
Fill out this short form, which will give you a signup code:

<https://lexe.app/dev-signup>

Download the mobile app for:

* iOS (Testflight): <https://lexe.app/beta-ios>
* Android (Play Store): <https://lexe.app/beta-android>

_iOS users_: Testflight sometimes asks for an "access code", which is different
from our signup code; you may need to reinstall the Testflight app to get past
this.

After installing the app, select "Create a new wallet", enter the signup code
sent to your email, connect your Google Drive (for backups), and set a backup
password. This will create a new self-custody Bitcoin+Lightning wallet that runs
in a secure enclave in the cloud; you can now send and receive Lightning
payments 24/7!

### Export client credentials for the SDK

To control your wallet using the Lexe Rust SDK, you'll first need to export
client credentials from the app:

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

If you want to use root-seed based auth or signup your own users to Lexe,
this is a bit more involved. Please get in touch via the signup form.

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
lexe = { git = "https://github.com/lexe-app/lexe-public" }
```

Basic usage:

```rust
use std::str::FromStr;

use lexe::{
    config::WalletEnvConfig,
    types::{
        auth::{Credentials, RootSeed},
        bitcoin::LxInvoice,
        command::{SdkCreateInvoiceRequest, SdkPayInvoiceRequest},
    },
    wallet::LexeWallet,
};

// Create a wallet config for mainnet (or testnet3() for testing)
let env_config = WalletEnvConfig::mainnet();

// Load root seed from ~/.lexe, or create a fresh one
let is_new_seed;
let root_seed = match env_config.read_seed()? {
    Some(seed) => {
        is_new_seed = false;
        seed
    }
    None => {
        is_new_seed = true;
        RootSeed::generate()
    }
};
let credentials = Credentials::RootSeed(root_seed.clone());

// Load or create wallet (data stored in ~/.lexe)
let wallet = LexeWallet::load_or_fresh(
    env_config.clone(),
    credentials.as_ref(),
    None, // Uses ~/.lexe by default
)?;

if is_new_seed {
    // Signup with Lexe and provision the node (idempotent)
    let partner_pk = None;
    wallet.signup(&root_seed, partner_pk).await?;
    env_config.write_seed(&root_seed)?;
} else {
    // Ensure provisioned to latest trusted release
    wallet.provision(credentials.as_ref()).await?;
}

// Get node info
let node_info = wallet.node_info().await?;
println!("Balance: {} sats", node_info.balance);

// Create a Lightning invoice
let invoice_req = SdkCreateInvoiceRequest {
    expiration_secs: 3600,
    amount: None,
    description: Some("VPN subscription (1 month)".to_string()),
    payer_note: None,
};
let invoice_resp = wallet.create_invoice(invoice_req).await?;

// Pay an invoice
let invoice = LxInvoice::from_str("lnbc1pjlue...")?;
let pay_req = SdkPayInvoiceRequest {
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

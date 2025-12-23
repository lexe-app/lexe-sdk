# Lexe Rust SDK

## Overview

The Lexe Rust SDK provides a native Rust interface for developers to control their
self-custodial, always-online [Lexe](https://lexe.app) Lightning node.

This repo contains public-facing docs and examples for the Lexe Rust SDK.
The `sdk-rust` source code can be found in the Lexe monorepo at:
<https://github.com/lexe-app/lexe-public/tree/master/sdk-rust>

## Quickstart

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

Add `lexe-sdk` to your `Cargo.toml`:

```toml
[dependencies]
lexe-sdk = { package = "sdk-rust", git = "https://github.com/lexe-app/lexe-public" }
```

Basic usage:

```rust
use lexe_sdk::{
    config::WalletEnvConfig,
    types::{Credentials, RootSeed, SysRng},
    wallet::LexeWallet,
};

// Initialize wallet
let mut rng = SysRng::new();
let env_config = WalletEnvConfig::prod();
let wallet = LexeWallet::load_or_fresh(
    &mut rng,
    env_config,
    credentials.as_ref(),
    lexe_data_dir,
)?;

// Ensure the node is provisioned to the latest version
wallet.ensure_provisioned(credentials.as_ref(), false, None, None).await?;

// Get node info
let node_info = wallet.node_info().await?;
println!("Balance: {} sats", node_info.balance);

// Sync payments from the node
let summary = wallet.sync_payments().await?;
println!("Synced {} new payments", summary.num_new);
```

Run the example:

```bash
cd rust-example
cargo run
```

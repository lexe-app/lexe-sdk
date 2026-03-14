# Lexe Python SDK

The Lexe Python SDK provides a native Python interface for developers to control
their self-custodial, always-online [Lexe](https://lexe.app) Lightning node.

```bash
pip install lexe-sdk
```

This repo contains public-facing docs and examples for the Lexe Python SDK.
The SDK source code can be found in the Lexe monorepo at:
<https://github.com/lexe-app/lexe-public/tree/master/sdk-uniffi>

# Quickstart

### Download the Lexe Wallet app

To get started, you'll need a Lexe wallet. Developers building on the Python
SDK are eligible for early access to Lexe wallet, which is currently in closed
beta. Fill out this short form, which will give you a signup code:

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

To control your wallet using the Lexe Python SDK, you'll first need to export
client credentials from the app:

1. Open the Lexe app > Menu sidebar > "SDK clients" > "Create new client"
2. Copy the client credentials string

Set `LEXE_CLIENT_CREDENTIALS` in your environment or in a `.env` file:

```bash
# Option 1: Set directly in environment
$ export LEXE_CLIENT_CREDENTIALS="eyJsZXhlX2F1dGhfdG9rZ...TA0In0"

# Option 2: Use a .env file
$ echo 'LEXE_CLIENT_CREDENTIALS=eyJsZXhlX2F1dGhfdG9rZ...TA0In0' > .env
$ chmod 600 .env
```

If you want to use root-seed based auth or signup your own users to Lexe,
this is a bit more involved. Please get in touch via the signup form.

### Install the SDK

```bash
pip install lexe-sdk
```

The package includes precompiled native bindings for Linux (x86_64, aarch64),
macOS (x86_64, Apple Silicon), and Windows (x86_64). Python 3.10+ is required.

### Create a new node for your user

Wallet developers can programmatically create Lexe nodes for their users. Each
user gets a self-custodial Lightning node running in a secure enclave.

```python
from lexe import LexeWallet, RootSeed, SeedFileError, WalletEnvConfig

# Create a wallet config for mainnet (or testnet3() for testing)
config = WalletEnvConfig.mainnet()

# Try to load an existing seed from ~/.lexe, or create a fresh one
try:
    seed = RootSeed.read(config)
    is_new_seed = False
except SeedFileError.NotFound:
    seed = RootSeed.generate()
    is_new_seed = True

# Load or create wallet (data stored in ~/.lexe by default)
wallet = LexeWallet.load_or_fresh(config, seed)

if is_new_seed:
    # Sign up the user and provision their node.
    # Pass your partner_user_pk to associate the node with your platform.
    wallet.signup(
        root_seed=seed,
        partner_pk=None,  # Your partner user_pk (hex), if applicable
    )

    # Persist the seed so we can load it on subsequent runs.
    # Stored at ~/.lexe/seedphrase.txt (mainnet).
    seed.write(config)
else:
    # Ensure the node is running the latest enclave version
    wallet.provision(seed)

# The node is now live. Query its info
info = wallet.node_info()
print(f"Node created! Public key: {info.node_pk}")
print(f"Balance: {info.balance_sats} sats")
```

### Using the SDK

Once a node is provisioned, you can create invoices, send payments, and more:

```python
from lexe import LexeWallet, PaymentFilter, RootSeed, WalletEnvConfig

config = WalletEnvConfig.mainnet()
seed = RootSeed.read(config)
wallet = LexeWallet.load_or_fresh(config, seed)
wallet.provision(seed)

# Get node info
info = wallet.node_info()
print(f"Balance: {info.balance_sats} sats")

# Create a Lightning invoice
invoice = wallet.create_invoice(
    expiration_secs=3600,
    amount_sats=None,
    description="Payment for coffee",
)
print(f"Invoice: {invoice.invoice}")

# Pay a Lightning invoice
payment = wallet.pay_invoice(
    invoice="lnbc...",
    fallback_amount_sats=None,
    note="Paying for coffee",
)

# Sync and list payments
wallet.sync_payments()
payments = wallet.list_payments(filter=PaymentFilter.ALL)
for p in payments.payments:
    print(f"  {p.status}: {p.amount_sats} sats")
```

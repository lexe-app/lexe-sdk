//! # Rust SDK Example
//!
//! This file only contains test code; see `main.rs` for example code.

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use lexe::{
        bip39::Mnemonic,
        config::{
            DeployEnv, Network, WalletEnv, WalletEnvConfig, WalletEnvDbConfig,
            WalletUserConfig, WalletUserDbConfig,
        },
        types::{
            auth::{
                ClientCredentials, Credentials, CredentialsRef, Measurement,
                NodePk, RootSeed, UserPk,
            },
            bitcoin::{Amount, ConfirmationPriority, Invoice, Txid},
            command::{
                CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentRequest,
                GetPaymentResponse, ListPaymentsResponse, NodeInfo,
                PayInvoiceRequest, PayInvoiceResponse, PaymentSyncSummary,
                UpdatePaymentNoteRequest,
            },
            payment::{
                ClientPaymentId, LnClaimId, Order, Payment,
                PaymentCreatedIndex, PaymentDirection, PaymentFilter,
                PaymentHash, PaymentId, PaymentKind, PaymentRail,
                PaymentSecret, PaymentStatus,
            },
            util::TimestampMs,
        },
        util::ByteArray,
        wallet::LexeWallet,
    };

    /// Test that the SDK reexports all types required to call all functions
    /// and handle all results of all stable APIs, by writing fake code which
    /// references all of those types; in other words, all types can be named.
    ///
    /// This test is `#[ignore]`d because it can't actually run - it just needs
    /// to compile to verify all types are properly exported.
    #[ignore]
    #[test]
    fn test_stable_apis_nameable() {
        #![allow(
            dead_code,
            unreachable_code,
            unused_variables,
            clippy::diverging_sub_expression
        )]

        // --- Config types ---
        let wallet_env: WalletEnv = WalletEnv::mainnet();
        let _: WalletEnv = WalletEnv::testnet3();
        let _: WalletEnv = WalletEnv::regtest(false);
        let _: DeployEnv = wallet_env.deploy_env;
        let _: Network = wallet_env.network;
        let _: bool = wallet_env.use_sgx;
        let _env_config: WalletEnvConfig = WalletEnvConfig::mainnet();
        let _env_config: WalletEnvConfig = WalletEnvConfig::testnet3();

        // --- Credential types ---
        let root_seed: &RootSeed = todo!();
        let client_creds: ClientCredentials = todo!();
        let credentials: Credentials = Credentials::RootSeed(todo!());
        let credentials: Credentials =
            Credentials::ClientCredentials(client_creds);
        let credentials_ref: CredentialsRef<'_> = credentials.as_ref();

        // --- Seed file I/O ---
        let data_dir: PathBuf = lexe::default_lexe_data_dir().unwrap();
        let env_config: WalletEnvConfig = WalletEnvConfig::mainnet();
        // Seedphrase path
        let _seedphrase_path: PathBuf = env_config.seedphrase_path(&data_dir);
        let _seedphrase_path: PathBuf =
            env_config.wallet_env.seedphrase_path(&data_dir);
        // RootSeed convenience I/O (resolves default ~/.lexe path)
        let _root_seed: Option<RootSeed> =
            RootSeed::read(&env_config.wallet_env).unwrap();
        let _: () = root_seed.write(&env_config.wallet_env).unwrap();
        // RootSeed path-based I/O
        let _root_seed: Option<RootSeed> =
            RootSeed::read_from_path(&_seedphrase_path).unwrap();
        let _: () = root_seed.write_to_path(&_seedphrase_path).unwrap();
        let mnemonic: Mnemonic = root_seed.to_mnemonic();
        let _root_seed: RootSeed = RootSeed::from_mnemonic(mnemonic).unwrap();
        let _: &[u8] = root_seed.as_bytes();
        let _: String = root_seed.to_hex();
        let _root_seed: RootSeed =
            RootSeed::from_bytes(root_seed.as_bytes()).unwrap();
        let _root_seed: RootSeed =
            RootSeed::from_hex(&root_seed.to_hex()).unwrap();
        let user_pk: UserPk = root_seed.derive_user_pk();
        let _: [u8; 32] = user_pk.to_array();
        let _: &[u8] = user_pk.as_slice();
        let _: String = user_pk.to_hex();
        let node_pk: NodePk = root_seed.derive_node_pk();
        let _: String = node_pk.to_hex();
        let encrypted: Vec<u8> =
            root_seed.password_encrypt("password").unwrap();
        let _root_seed: RootSeed =
            RootSeed::password_decrypt("password", encrypted).unwrap();

        // --- DB config types ---
        let env_db_config: WalletEnvDbConfig =
            WalletEnvDbConfig::new(wallet_env, data_dir.clone());
        let _: &PathBuf = env_db_config.lexe_data_dir();
        let _: &PathBuf = env_db_config.env_db_dir();
        let user_db_config: WalletUserDbConfig =
            WalletUserDbConfig::new(env_db_config.clone(), user_pk);
        let _user_db_config: WalletUserDbConfig =
            WalletUserDbConfig::from_credentials(
                credentials_ref,
                env_db_config,
            )
            .unwrap();
        let _: &WalletEnvDbConfig = user_db_config.env_db_config();
        let _: UserPk = user_db_config.user_pk();
        let _: &PathBuf = user_db_config.lexe_data_dir();
        let _: &PathBuf = user_db_config.env_db_dir();
        let _: &PathBuf = user_db_config.user_db_dir();

        // --- LexeWallet constructors ---
        let wallet: LexeWallet = LexeWallet::fresh(
            env_config.clone(),
            credentials_ref,
            Some(data_dir.clone()),
        )
        .unwrap();
        let _wallet: Option<LexeWallet> = LexeWallet::load(
            env_config.clone(),
            credentials_ref,
            Some(data_dir.clone()),
        )
        .unwrap();
        let wallet: LexeWallet = LexeWallet::load_or_fresh(
            env_config.clone(),
            credentials_ref,
            Some(data_dir),
        )
        .unwrap();

        let _wallet_without_db: LexeWallet =
            LexeWallet::without_db(env_config.clone(), credentials_ref)
                .unwrap();

        // --- LexeWallet DB methods ---

        async fn test_wallet_db_async(wallet: &LexeWallet) {
            let summary: PaymentSyncSummary =
                wallet.sync_payments().await.unwrap();
            let _: usize = summary.num_new;
            let _: usize = summary.num_updated;

            let resp: ListPaymentsResponse = wallet
                .list_payments(&PaymentFilter::All, None, None, None)
                .unwrap();
            let _: Vec<Payment> = resp.payments;
            let _: Option<PaymentCreatedIndex> = resp.next_index;

            // Test all filter variants
            let _ = wallet.list_payments(
                &PaymentFilter::Pending,
                Some(Order::Asc),
                Some(10),
                None,
            );
            let _ = wallet.list_payments(
                &PaymentFilter::Completed,
                None,
                None,
                None,
            );
            let _ =
                wallet.list_payments(&PaymentFilter::Failed, None, None, None);
            let _ = wallet.list_payments(
                &PaymentFilter::Finalized,
                None,
                None,
                None,
            );

            wallet.clear_payments().unwrap();

            // wait_for_payment
            let index: PaymentCreatedIndex = todo!();
            let _: Payment =
                wallet.wait_for_payment(index, None).await.unwrap();
        }

        // --- LexeWallet shared methods ---
        let user_config: &WalletUserConfig = wallet.user_config();
        let _: UserPk = user_config.user_pk;
        let _: WalletEnvConfig = user_config.env_config.clone();

        async fn test_wallet_async(wallet: &LexeWallet) {
            // node_info
            let info: NodeInfo = wallet.node_info().await.unwrap();
            let _: Measurement = info.measurement;
            let _: String = info.measurement.to_hex();
            let _: UserPk = info.user_pk;
            let _: NodePk = info.node_pk;
            let _: Amount = info.balance;
            let _: Amount = info.lightning_balance;

            // create_invoice
            let req = CreateInvoiceRequest {
                expiration_secs: 3600,
                amount: None,
                description: None,
                payer_note: None,
            };
            let resp: CreateInvoiceResponse =
                wallet.create_invoice(req).await.unwrap();
            let _: PaymentCreatedIndex = resp.index;
            let _: TimestampMs = resp.created_at;
            let _: TimestampMs = resp.expires_at;
            let _: PaymentHash = resp.payment_hash;
            let _: PaymentSecret = resp.payment_secret;

            // pay_invoice
            let invoice: Invoice = todo!();
            let req = PayInvoiceRequest {
                invoice,
                fallback_amount: None,
                note: Some("Test payment".to_string()),
                payer_note: None,
            };
            let resp: PayInvoiceResponse =
                wallet.pay_invoice(req).await.unwrap();
            let _: PaymentCreatedIndex = resp.index;
            let _: TimestampMs = resp.created_at;

            // get_payment
            let req: GetPaymentRequest = GetPaymentRequest { index: todo!() };
            let resp: GetPaymentResponse =
                wallet.get_payment(req).await.unwrap();
            let payment: Payment = resp.payment.unwrap();
            let _: PaymentCreatedIndex = payment.index;
            let _: PaymentId = payment.index.id;
            // PaymentId variant payload types
            let _: ClientPaymentId = todo!();
            let _: LnClaimId = todo!();
            let _: PaymentRail = payment.rail;
            let _: PaymentKind = payment.kind;
            let _: PaymentDirection = payment.direction;
            let _: PaymentStatus = payment.status;
            let _: Amount = payment.fees;
            let _: TimestampMs = payment.created_at;
            let _: TimestampMs = payment.updated_at;
            let _: Option<Txid> = payment.txid;
            let _: Option<ConfirmationPriority> = payment.priority;

            // update_payment_note
            let req: UpdatePaymentNoteRequest = todo!();
            wallet.update_payment_note(req).await.unwrap();
        }

        async fn test_signup(wallet: &LexeWallet, root_seed: &RootSeed) {
            let partner_pk: Option<UserPk> = None;
            wallet.signup(root_seed, partner_pk).await.unwrap();
        }

        async fn test_provision(wallet: &LexeWallet) {
            let credentials_ref: CredentialsRef<'_> = todo!();
            wallet.provision(credentials_ref).await.unwrap();
        }
    }
}

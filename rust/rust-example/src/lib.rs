//! # Rust SDK Example
//!
//! This file only contains test code; see `main.rs` for example code.

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use lexe::{
        bip39::Mnemonic,
        config::{WalletEnvConfig, WalletUserConfig},
        types::{
            auth::{
                ClientCredentials, Credentials, CredentialsRef, Measurement,
                NodePk, RootSeed, UserPk,
            },
            bitcoin::{Amount, ConfirmationPriority, LxInvoice, LxTxid},
            command::{
                CreateInvoiceRequest, CreateInvoiceResponse, GetPaymentRequest,
                GetPaymentResponse, ListPaymentsResponse, NodeInfo,
                PayInvoiceRequest, PayInvoiceResponse, PaymentSyncSummary,
                UpdatePaymentNoteRequest,
            },
            payment::{
                LxPaymentHash, LxPaymentId, LxPaymentSecret, Order, Payment,
                PaymentCreatedIndex, PaymentDirection, PaymentFilter,
                PaymentKind, PaymentRail, PaymentStatus,
            },
            util::TimestampMs,
        },
        util::ByteArray,
        wallet::{LexeWallet, WithDb, WithoutDb},
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
        let encrypted: Vec<u8> = root_seed.password_encrypt("password").unwrap();
        let _root_seed: RootSeed =
            RootSeed::password_decrypt("password", encrypted).unwrap();

        // --- LexeWallet constructors ---
        // LexeWallet<WithDb>
        let wallet_with_db: LexeWallet<WithDb> = LexeWallet::fresh(
            env_config.clone(),
            credentials_ref,
            Some(data_dir.clone()),
        )
        .unwrap();
        let wallet_with_db: Option<LexeWallet<WithDb>> = LexeWallet::load(
            env_config.clone(),
            credentials_ref,
            Some(data_dir.clone()),
        )
        .unwrap();
        let wallet_with_db: LexeWallet<WithDb> = LexeWallet::load_or_fresh(
            env_config.clone(),
            credentials_ref,
            Some(data_dir),
        )
        .unwrap();

        // LexeWallet<WithoutDb>
        let _wallet_without_db: LexeWallet<WithoutDb> =
            LexeWallet::without_db(env_config.clone(), credentials_ref)
                .unwrap();

        // --- LexeWallet<WithDb> methods ---

        async fn test_wallet_with_db_async(
            wallet: &LexeWallet<lexe::wallet::WithDb>,
        ) {
            let summary: PaymentSyncSummary =
                wallet.sync_payments().await.unwrap();
            let _: usize = summary.num_new;
            let _: usize = summary.num_updated;

            let resp: ListPaymentsResponse =
                wallet.list_payments(&PaymentFilter::All, None, None, None);
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

        // --- LexeWallet<D> generic methods ---
        let user_config: &WalletUserConfig = wallet_with_db.user_config();
        let _: UserPk = user_config.user_pk;
        let _: WalletEnvConfig = user_config.env_config.clone();

        async fn test_wallet_generic_async<D>(wallet: &LexeWallet<D>) {
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
            let _: LxPaymentHash = resp.payment_hash;
            let _: LxPaymentSecret = resp.payment_secret;

            // pay_invoice
            let invoice: LxInvoice = todo!();
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
            let _: LxPaymentId = payment.index.id;
            let _: PaymentRail = payment.rail;
            let _: PaymentKind = payment.kind;
            let _: PaymentDirection = payment.direction;
            let _: PaymentStatus = payment.status;
            let _: Amount = payment.fees;
            let _: TimestampMs = payment.created_at;
            let _: TimestampMs = payment.updated_at;
            let _: Option<LxTxid> = payment.txid;
            let _: Option<ConfirmationPriority> = payment.priority;

            // update_payment_note
            let req: UpdatePaymentNoteRequest = todo!();
            wallet.update_payment_note(req).await.unwrap();
        }

        async fn test_signup<D>(wallet: &LexeWallet<D>, root_seed: &RootSeed) {
            let partner_pk: Option<UserPk> = None;
            wallet.signup(root_seed, partner_pk).await.unwrap();
        }

        async fn test_provision<D>(wallet: &LexeWallet<D>) {
            let credentials_ref: CredentialsRef<'_> = todo!();
            wallet.provision(credentials_ref).await.unwrap();
        }
    }
}

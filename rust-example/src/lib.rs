//! # Rust SDK Example
//!
//! This file only contains test code; see `main.rs` for example code.

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use lexe_sdk::{
        config::{WalletEnvConfig, WalletUserConfig},
        payments_db::{PaymentSyncSummary, PaymentsDb},
        types::{
            BasicPaymentV2, ClientCredentials, Credentials, CredentialsRef,
            PaymentCreatedIndex, PaymentUpdatedIndex, RootSeed,
            SdkCreateInvoiceRequest, SdkCreateInvoiceResponse,
            SdkGetPaymentRequest, SdkGetPaymentResponse, SdkNodeInfo,
            SdkPayInvoiceRequest, SdkPayInvoiceResponse, SdkPayment, SysRng,
            UpdatePaymentNote, UserPk,
        },
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
        let _env_config: WalletEnvConfig = WalletEnvConfig::prod();
        let _env_config: WalletEnvConfig = WalletEnvConfig::staging();
        let _env_config: WalletEnvConfig =
            WalletEnvConfig::dev(false, None::<&str>);

        // --- Credential types ---
        let rng: &mut SysRng = todo!();
        let root_seed: &RootSeed = todo!();
        let client_creds: ClientCredentials = todo!();
        let credentials: Credentials = Credentials::RootSeed(todo!());
        let credentials: Credentials =
            Credentials::ClientCredentials(client_creds);
        let credentials_ref: CredentialsRef<'_> = credentials.as_ref();

        // --- LexeWallet constructors ---
        let env_config: WalletEnvConfig = todo!();
        let lexe_data_dir: PathBuf = todo!();

        // LexeWallet<WithDb>
        let wallet_with_db: LexeWallet<WithDb> = LexeWallet::fresh(
            rng,
            env_config.clone(),
            credentials_ref,
            lexe_data_dir.clone(),
        )
        .unwrap();
        let wallet_with_db: Option<LexeWallet<WithDb>> = LexeWallet::load(
            rng,
            env_config.clone(),
            credentials_ref,
            lexe_data_dir.clone(),
        )
        .unwrap();
        let wallet_with_db: LexeWallet<WithDb> = LexeWallet::load_or_fresh(
            rng,
            env_config.clone(),
            credentials_ref,
            lexe_data_dir,
        )
        .unwrap();

        // LexeWallet<WithoutDb>
        let _wallet_without_db: LexeWallet<WithoutDb> =
            LexeWallet::without_db(rng, env_config.clone(), credentials_ref)
                .unwrap();

        // --- LexeWallet<WithDb> methods ---
        let payments_db: &PaymentsDb<_> = wallet_with_db.payments_db();

        async fn test_wallet_with_db_async(
            wallet: &LexeWallet<lexe_sdk::wallet::WithDb>,
        ) {
            let summary: PaymentSyncSummary =
                wallet.sync_payments().await.unwrap();
            let _: usize = summary.num_new;
            let _: usize = summary.num_updated;
            let _: bool = summary.any_changes();
        }

        // --- LexeWallet<D> generic methods ---
        let user_config: &WalletUserConfig = wallet_with_db.user_config();
        let _: UserPk = user_config.user_pk;
        let _: WalletEnvConfig = user_config.env_config.clone();

        async fn test_wallet_generic_async<D>(wallet: &LexeWallet<D>) {
            // node_info
            let _node_info: SdkNodeInfo = wallet.node_info().await.unwrap();

            // create_invoice
            let req = SdkCreateInvoiceRequest {
                expiration_secs: 3600,
                amount: None,
                description: None,
            };
            let resp: SdkCreateInvoiceResponse =
                wallet.create_invoice(req).await.unwrap();
            let _: PaymentCreatedIndex = resp.index;

            // pay_invoice
            let req: SdkPayInvoiceRequest = todo!();
            let resp: SdkPayInvoiceResponse =
                wallet.pay_invoice(req).await.unwrap();
            let _: PaymentCreatedIndex = resp.index;

            // get_payment
            let req: SdkGetPaymentRequest =
                SdkGetPaymentRequest { index: todo!() };
            let resp: SdkGetPaymentResponse =
                wallet.get_payment(req).await.unwrap();
            let _: Option<SdkPayment> = resp.payment;

            // update_payment_note
            let req: UpdatePaymentNote = todo!();
            wallet.update_payment_note(req).await.unwrap();
        }

        async fn test_signup_and_provision<D>(
            wallet: &LexeWallet<D>,
            rng: &mut SysRng,
            root_seed: &RootSeed,
        ) {
            let partner: Option<UserPk> = None;
            let signup_code: Option<String> = None;
            let allow_gvfs_access = false;
            let backup_password: Option<&str> = None;
            let google_auth_code: Option<String> = None;

            wallet
                .signup_and_provision(
                    rng,
                    root_seed,
                    partner,
                    signup_code,
                    allow_gvfs_access,
                    backup_password,
                    google_auth_code,
                )
                .await
                .unwrap();
        }

        async fn test_ensure_provisioned<D>(wallet: &LexeWallet<D>) {
            let credentials_ref: CredentialsRef<'_> = todo!();
            let allow_gvfs_access = false;
            let encrypted_seed: Option<Vec<u8>> = None;
            let google_auth_code: Option<String> = None;

            wallet
                .ensure_provisioned(
                    credentials_ref,
                    allow_gvfs_access,
                    encrypted_seed,
                    google_auth_code,
                )
                .await
                .unwrap();
        }

        // --- PaymentsDb methods ---
        // Test PaymentsDb methods using a closure that captures the type
        let test_payments_db = |db: &PaymentsDb<_>| {
            let _ = db.delete();
            let _: usize = db.num_payments();
            let _: usize = db.num_pending();
            let _: usize = db.num_finalized();
            let _: usize = db.num_pending_not_junk();
            let _: usize = db.num_finalized_not_junk();
            let _: Option<PaymentUpdatedIndex> = db.latest_updated_index();

            let created_index: PaymentCreatedIndex = todo!();
            let _: Option<BasicPaymentV2> =
                db.get_payment_by_created_index(&created_index);
            let _: Option<BasicPaymentV2> = db.get_payment_by_scroll_idx(0);
            let _: Option<BasicPaymentV2> =
                db.get_pending_payment_by_scroll_idx(0);
            let _: Option<BasicPaymentV2> =
                db.get_pending_not_junk_payment_by_scroll_idx(0);
            let _: Option<BasicPaymentV2> =
                db.get_finalized_payment_by_scroll_idx(0);
            let _: Option<BasicPaymentV2> =
                db.get_finalized_not_junk_payment_by_scroll_idx(0);
        };
        test_payments_db(payments_db);
    }
}

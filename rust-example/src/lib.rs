//! # Rust SDK Example
//!
//! This file only contains test code; see `main.rs` for example code.

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    use lexe::{
        bip39::Mnemonic,
        bitcoin::{
            address::{Address, NetworkUnchecked},
            Transaction,
        },
        config::{
            DeployEnv, Network, WalletEnv, WalletEnvConfig, WalletEnvDbConfig,
            WalletUserConfig, WalletUserDbConfig,
        },
        semver::Version,
        types::{
            auth::{
                ClientCredentials, Credentials, CredentialsRef, Measurement,
                NodePk, RootSeed, UserPk,
            },
            bitcoin::{
                Amount, ChannelId, ClaimMethod, ConfirmationPriority, Invoice,
                LnurlPayRequest, LnurlPayRequestMetadata, LnurlWithdrawRequest,
                Offer, OutPoint, PaymentMethod, Txid, UserChannelId,
            },
            command::{
                AnalyzeRequest, AnalyzeResponse, ChannelDetails,
                ClaimableDetails, ClientInfo, ClientInfoResponse,
                CloseChannelRequest, CreateClientRequest, CreateClientResponse,
                CreateInvoiceRequest, CreateInvoiceResponse,
                CreateOfferRequest, CreateOfferResponse, GetPaymentRequest,
                GetPaymentResponse, GetUpdatedPaymentsRequest,
                GetUpdatedPaymentsResponse, ListChannelsResponse,
                ListClientsResponse, ListPaymentsResponse, NodeInfo,
                OpenChannelRequest, OpenChannelResponse, PayInvoiceRequest,
                PayLnurlRequest, PayOfferRequest, PayRequest, PayableDetails,
                PaymentSyncSummary, RevokeClientRequest, UpdateClientRequest,
                UpdatePersonalNoteRequest, WithdrawLnurlRequest,
            },
            payment::{
                ClientPaymentId, LnClaimId, OfferId, Order, Payment,
                PaymentCreatedIndex, PaymentDirection, PaymentFilter,
                PaymentHash, PaymentId, PaymentKind, PaymentPreimage,
                PaymentRail, PaymentSecret, PaymentStatus, PaymentUpdatedIndex,
            },
            util::{Ppm, TimestampMs},
        },
        util::{ed25519, ByteArray},
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

        // --- Crypto types ---
        let pubkey: ed25519::PublicKey = todo!();
        let _: String = pubkey.to_string();
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
            let PaymentSyncSummary {
                num_new,
                num_updated,
            } = wallet.sync_payments().await.unwrap();
            let _: usize = num_new;
            let _: usize = num_updated;

            let ListPaymentsResponse {
                payments,
                next_index,
            } = wallet
                .list_payments(&PaymentFilter::All, None, None, None)
                .unwrap();
            let _: Vec<Payment> = payments;
            let _: Option<PaymentCreatedIndex> = next_index;

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
            let NodeInfo {
                version,
                measurement,
                user_pk,
                node_pk,
                balance,
                lightning_balance,
                lightning_sendable_balance,
                lightning_max_sendable_balance,
                onchain_balance,
                onchain_trusted_balance,
                num_channels,
                num_usable_channels,
            } = wallet.node_info().await.unwrap();
            let _: Version = version;
            let _: Measurement = measurement;
            let _: String = measurement.to_hex();
            let _: UserPk = user_pk;
            let _: NodePk = node_pk;
            let _: Amount = balance;
            let _: Amount = lightning_balance;
            let _: Amount = lightning_sendable_balance;
            let _: Amount = lightning_max_sendable_balance;
            let _: Amount = onchain_balance;
            let _: Amount = onchain_trusted_balance;
            let _: usize = num_channels;
            let _: usize = num_usable_channels;

            // list_channels
            let ListChannelsResponse { channels } =
                wallet.list_channels().await.unwrap();
            let ChannelDetails {
                channel_id,
                user_channel_id,
                funding_txo,
                is_usable,
                channel_value,
                our_balance,
                their_balance,
                punishment_reserve,
                outbound_capacity,
                inbound_capacity,
            } = channels.into_iter().next().unwrap();
            let _: ChannelId = channel_id;
            let _: UserChannelId = user_channel_id;
            let _: Option<OutPoint> = funding_txo;
            let _: bool = is_usable;
            let _: Amount = channel_value;
            let _: Amount = our_balance;
            let _: Amount = their_balance;
            let _: Amount = punishment_reserve;
            let _: Amount = outbound_capacity;
            let _: Amount = inbound_capacity;

            // open_channel
            let req = OpenChannelRequest {
                value: Amount::from_sats_u32(1_000_000),
                user_channel_id: None,
            };
            let OpenChannelResponse {
                channel_id,
                user_channel_id,
            } = wallet.open_channel(req).await.unwrap();
            let _: ChannelId = channel_id;
            let _: UserChannelId = user_channel_id;

            // close_channel
            let req = CloseChannelRequest { channel_id };
            wallet.close_channel(req).await.unwrap();

            // analyze
            let req = AnalyzeRequest {
                payment_string: "lnondeezn".to_owned(),
            };
            let AnalyzeResponse {
                payables,
                claimables,
            } = wallet.analyze(req).await.unwrap();
            // payables
            let PayableDetails {
                payable,
                method,
                description,
                amount,
                min_amount,
                max_amount,
                expires_at,
            } = payables.into_iter().next().unwrap();
            let _: String = payable;
            let _: Option<String> = description;
            let _: Option<Amount> = amount;
            let _: Option<Amount> = min_amount;
            let _: Option<Amount> = max_amount;
            let _: Option<TimestampMs> = expires_at;
            let _: &'static str = method.kind();
            match method {
                PaymentMethod::Onchain {
                    address,
                    amount,
                    label,
                    message,
                } => {
                    let _: Address = address;
                    let _: Option<Amount> = amount;
                    let _: Option<String> = label;
                    let _: Option<String> = message;
                }
                PaymentMethod::Invoice { invoice } => {
                    let _: Invoice = invoice;
                }
                PaymentMethod::Offer {
                    offer,
                    bip321_amount,
                    human_bitcoin_address,
                } => {
                    let _: Offer = offer;
                    let _: Option<Amount> = bip321_amount;
                    let _: Option<String> = human_bitcoin_address;
                }
                PaymentMethod::LnurlPay {
                    pay_request,
                    lnurl: _,
                    lightning_address,
                } => {
                    let _: LnurlPayRequest = pay_request;
                    let _: LnurlPayRequestMetadata = pay_request.metadata;
                    let _: Option<String> = lightning_address;
                }
            };
            // claimables
            let ClaimableDetails {
                claimable,
                method,
                description,
                min_amount,
                max_amount,
            } = claimables.into_iter().next().unwrap();
            let _: String = claimable;
            let _: Option<String> = description;
            let _: Option<Amount> = min_amount;
            let _: Option<Amount> = max_amount;
            match method {
                ClaimMethod::LnurlWithdraw {
                    lnurl: _,
                    withdraw_request,
                } => {
                    let _: LnurlWithdrawRequest = withdraw_request;
                }
            }

            // pay
            let req = PayRequest {
                payable: "lnondeenz".to_string(),
                amount: None,
                message: None,
                personal_note: None,
            };
            let _: Payment = wallet.pay(req).await.unwrap();

            // create_invoice
            let req = CreateInvoiceRequest {
                expiration_secs: Some(3600),
                amount: None,
                description: None,
                personal_note: None,
                partner_pk: None,
                partner_prop_fee: None,
                partner_base_fee: None,
            };
            let CreateInvoiceResponse {
                index,
                invoice,
                description,
                amount,
                created_at,
                expires_at,
                payment_hash,
                payment_secret,
            } = wallet.create_invoice(req).await.unwrap();
            let _: PaymentCreatedIndex = index;
            let _: Invoice = invoice;
            let _: Option<String> = description;
            let _: Option<Amount> = amount;
            let _: TimestampMs = created_at;
            let _: TimestampMs = expires_at;
            let _: PaymentHash = payment_hash;
            let _: PaymentSecret = payment_secret;

            // pay_invoice
            let invoice: Invoice = todo!();
            let req = PayInvoiceRequest {
                invoice,
                fallback_amount: None,
                personal_note: Some("Test payment".to_string()),
            };
            let _: Payment = wallet.pay_invoice(req).await.unwrap();

            // create_offer
            let req = CreateOfferRequest {
                description: Some("Donations".to_string()),
                min_amount: None,
                expiration_secs: None,
            };
            let CreateOfferResponse { offer } =
                wallet.create_offer(req).await.unwrap();
            let _: Offer = offer;

            // pay_offer
            let offer: Offer = todo!();
            let req = PayOfferRequest {
                offer,
                amount: Amount::from_sats_u32(1000),
                message: None,
                personal_note: None,
            };
            let _: Payment = wallet.pay_offer(req).await.unwrap();

            // pay_lnurl
            let req = PayLnurlRequest {
                lnurl: Some("lnurl1...".to_string()),
                pay_request: None,
                amount: Amount::from_sats_u32(1000),
                message: None,
                personal_note: None,
            };
            let _: Payment = wallet.pay_lnurl(req).await.unwrap();

            // withdraw_lnurl
            let req = WithdrawLnurlRequest {
                lnurl: Some("lnurl1...".to_string()),
                withdraw_request: None,
                amount: Some(Amount::from_sats_u32(1000)),
                description: None,
                personal_note: None,
            };
            let _: Payment = wallet.withdraw_lnurl(req).await.unwrap();

            // get_payment
            let req: GetPaymentRequest = GetPaymentRequest { index: todo!() };
            let GetPaymentResponse { payment } =
                wallet.get_payment(req).await.unwrap();
            let Payment {
                index,
                rail,
                kind,
                direction,
                hash,
                preimage,
                offer_id,
                txid,
                amount,
                fees,
                partner_pk,
                partner_prop_fee,
                partner_base_fee,
                status,
                status_msg,
                address,
                invoice,
                tx,
                payer_name,
                message,
                personal_note,
                priority,
                expires_at,
                finalized_at,
                created_at,
                updated_at,
            } = payment.unwrap();
            let _: PaymentCreatedIndex = index;
            let _: PaymentId = index.id;
            let _: PaymentRail = rail;
            let _: PaymentKind = kind;
            let _: PaymentDirection = direction;
            let _: Option<PaymentHash> = hash;
            let _: Option<PaymentPreimage> = preimage;
            let _: Option<OfferId> = offer_id;
            let _: Option<Txid> = txid;
            let _: Option<Amount> = amount;
            let _: Amount = fees;
            let _: Option<UserPk> = partner_pk;
            let _: Option<Ppm> = partner_prop_fee;
            let _: Option<Amount> = partner_base_fee;
            let _: PaymentStatus = status;
            let _: String = status_msg;
            let _: Option<Arc<Address<NetworkUnchecked>>> = address;
            let _: Option<Arc<Invoice>> = invoice;
            let _: Option<Arc<Transaction>> = tx;
            let _: Option<String> = payer_name;
            let _: Option<String> = message;
            let _: Option<String> = personal_note;
            let _: Option<ConfirmationPriority> = priority;
            let _: Option<TimestampMs> = expires_at;
            let _: Option<TimestampMs> = finalized_at;
            let _: TimestampMs = created_at;
            let _: TimestampMs = updated_at;
            // PaymentId variant payload types
            let _: ClientPaymentId = todo!();
            let _: LnClaimId = todo!();

            // get_updated_payments
            let req: GetUpdatedPaymentsRequest = GetUpdatedPaymentsRequest {
                start_index: None,
                limit: None,
            };
            let GetUpdatedPaymentsResponse {
                payments,
                updated_index,
            } = wallet.get_updated_payments(req).await.unwrap();
            let _: Vec<Payment> = payments;
            let _: Option<PaymentUpdatedIndex> = updated_index;

            // update_personal_note
            let req: UpdatePersonalNoteRequest = todo!();
            wallet.update_personal_note(req).await.unwrap();

            // list_clients
            let ListClientsResponse { clients } =
                wallet.list_clients().await.unwrap();
            let clients: HashMap<ed25519::PublicKey, ClientInfo> = clients;
            let ClientInfo {
                client_pk,
                created_at,
                expires_at,
                label,
            } = clients.into_values().next().unwrap();
            let _: ed25519::PublicKey = client_pk;
            let _: TimestampMs = created_at;
            let _: Option<TimestampMs> = expires_at;
            let _: Option<String> = label;

            // create_client
            let req = CreateClientRequest {
                expires_at: None,
                label: Some("my-client".to_string()),
            };
            let CreateClientResponse {
                client_pk,
                client_credentials,
                created_at,
            } = wallet.create_client(req).await.unwrap();
            let _: ClientCredentials = client_credentials;
            let _: ed25519::PublicKey = client_pk;
            let _: TimestampMs = created_at;

            // update_client
            let req = UpdateClientRequest {
                client_pk,
                new_label: Some(Some("renamed-client".to_string())),
                new_expires_at: Some(None),
            };
            let ClientInfoResponse { client } =
                wallet.update_client(req).await.unwrap();
            let _: ClientInfo = client;

            // revoke_client
            let req = RevokeClientRequest { client_pk };
            let ClientInfoResponse { client } =
                wallet.revoke_client(req).await.unwrap();
            let _: ClientInfo = client;
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

use std::sync::Arc;
use tokio::{runtime::Runtime, sync::Mutex};

use walletka_core::{
    bdk::bitcoin::Network,
    builder::WalletkaBuilder as BuilderSdk,
    enums::{WalletkaAssetLocation, WalletkaAssetState, WalletkaLayer},
    types::{Amount, Currency, WalletkaAsset, WalletkaBalance},
    Walletka as WalletkaSdk,
};

mod utils;
use utils::{generate_mnemonic, validate_mnemonic};

uniffi::include_scaffolding!("walletka-core");

struct WalletkaBuilder {
    inner_builder: Mutex<BuilderSdk>,
    rt: Runtime,
}

impl WalletkaBuilder {
    fn new() -> Self {
        Self {
            inner_builder: Mutex::new(BuilderSdk::default()),
            rt: Runtime::new().unwrap(),
        }
    }

    fn set_mnemonic(&self, mnemonic_words: String) {
        self.rt.block_on(async {
            self.inner_builder
                .lock()
                .await
                .set_mnemonic(mnemonic_words, None)
        });
    }

    fn set_memory_db_store(&self) {
        self.rt.block_on(async {
            self.inner_builder.lock().await.set_memory_db_store();
        });
    }

    fn set_local_db_store(&self, data_path: String) {
        self.rt.block_on(async {
            self.inner_builder
                .lock()
                .await
                .set_local_db_store(data_path);
        });
    }

    fn set_data_path(&self, data_path: String) {
        self.rt.block_on(async {
            self.inner_builder.lock().await.set_data_path(data_path);
        });
    }

    fn set_network(&self, network: Network) {
        self.rt.block_on(async {
            self.inner_builder.lock().await.set_network(network);
        });
    }

    fn set_nostr_relays(&self, relays: Vec<String>) {
        self.rt.block_on(async {
            self.inner_builder.lock().await.set_nostr_relays(relays);
        });
    }

    fn set_electrum_url(&self, electrum_url: Option<String>) {
        self.rt.block_on(async {
            self.inner_builder
                .lock()
                .await
                .set_electrum_url(electrum_url);
        });
    }

    fn build(&self) -> Arc<Walletka> {
        let walletka_rt = Runtime::new().unwrap();
        let walletka = walletka_rt
            .block_on(async { self.inner_builder.lock().await.build().await.unwrap() });

        Arc::new(Walletka {
            inner_wallet: Mutex::new(walletka),
            rt: walletka_rt,
        })
    }
}

struct Walletka {
    inner_wallet: Mutex<WalletkaSdk>,
    rt: Runtime,
}

impl Walletka {
    async fn sync(&self, light: bool) {
        self.inner_wallet.lock().await.sync(light).await.unwrap();
    }

    fn get_bitcoin_address(&self) -> String {
        self.inner_wallet
            .blocking_lock()
            .get_bitcoin_address()
            .unwrap()
            .to_string()
    }

    fn get_balance(&self, currency_symbol: Option<String>) -> WalletkaBalance {
        self.rt.block_on(async {
            self.inner_wallet
                .lock()
                .await
                .get_balance(currency_symbol)
                .await
                .unwrap()
        })
    }

    fn get_assets(&self) -> Vec<WalletkaAsset> {
        let assets = self
            .rt
            .block_on(async { self.inner_wallet.lock().await.get_assets().await.unwrap() });

        assets
    }

    fn claim_cashu_token(&self, token: String) {
        self.rt.block_on(async {
            self.inner_wallet
                .lock()
                .await
                .claim_cashu_token(token)
                .await
                .unwrap();
        })
    }

    fn create_rgb_utxos(&self) {
        self.inner_wallet
            .blocking_lock()
            .create_rgb_utxos()
            .unwrap();
    }

    fn create_rgb_invoice(
        &self,
        asset_id: Option<String>,
        amount: Option<u64>,
        duration_seconds: Option<u32>,
        min_confirmations: Option<u8>,
        transport_url: Option<String>,
        blinded: bool,
    ) -> String {
        self.inner_wallet
            .blocking_lock()
            .create_rgb_invoice(
                asset_id,
                amount,
                duration_seconds,
                min_confirmations,
                transport_url,
                blinded,
            )
            .unwrap()
    }

    fn issue_rgb20_asset(
        &self,
        ticker: String,
        name: String,
        precision: u8,
        amount: u64,
    ) -> String {
        self.inner_wallet
            .blocking_lock()
            .issue_rgb20_asset(ticker, name, precision, amount)
            .unwrap()
    }
}

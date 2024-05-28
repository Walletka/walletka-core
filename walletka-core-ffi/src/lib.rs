use std::sync::Arc;
use tokio::{runtime::Runtime, sync::Mutex};

use walletka_core::{
    bdk::bitcoin::Network,
    builder::WalletkaBuilder as BuilderSdk,
    enums::{WalletkaLayer, WalletkaAssetState, WalletkaAssetLocation},
    types::{Amount, Currency, WalletkaAsset, WalletkaBalance},
    Walletka as WalletkaSdk,
};

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
        let walletka = self
            .rt
            .block_on(async { self.inner_builder.lock().await.build().await.unwrap() });

        Arc::new(Walletka {
            inner_waller: Mutex::new(walletka),
            rt: Runtime::new().unwrap(),
        })
    }
}

struct Walletka {
    inner_waller: Mutex<WalletkaSdk>,
    rt: Runtime,
}

impl Walletka {
    async fn sync(&self) {
        self.inner_waller.lock().await.sync().await.unwrap();
    }

    fn get_bitcoin_address(&self) -> String {
        self.rt.block_on(async {
            self.inner_waller
                .lock()
                .await
                .get_bitcoin_address()
                .unwrap()
                .to_string()
        })
    }

    fn get_balance(&self, currency_symbol: Option<String>) -> WalletkaBalance {
        self.rt.block_on(async {
            self.inner_waller
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
            .block_on(async { self.inner_waller.lock().await.get_assets().await.unwrap() });

        assets
    }
}

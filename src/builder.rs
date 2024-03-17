use anyhow::Result;
use bdk::bitcoin::Network;
use bdk::blockchain::{AnyBlockchain, ElectrumBlockchain};
use bdk::keys::bip39::Mnemonic;
use log::debug;
use surrealdb::engine::local::Db;

use crate::wallets::bitcoin::BitcoinWallet;
use crate::io::clients::NostrClient;
use crate::{
    io::{
        database::{get_database, DatabaseStore},
        repositories::contacts_repository::ContactsRepository,
    },
    services::ContactsService,
    Walletka,
};

pub struct WalletkaBuilder {
    pub wallet_id: Option<String>,
    pub database_store: DatabaseStore,
    pub network: Network,
    pub mnemonic_words: Option<String>,
    pub passphrase: Option<String>,
    pub data_path: String,
    pub nostr_relay_urls: Vec<String>,
    pub electrum_url: String,
    pub esplora_url: String,
}

// Todo Needed?
impl Default for WalletkaBuilder {
    fn default() -> Self {
        Self {
            wallet_id: Some(Network::Testnet.to_string()),
            database_store: DatabaseStore::Memory,
            network: Network::Testnet,
            mnemonic_words: None,
            passphrase: Default::default(),
            data_path: "".to_string(), // Todo
            nostr_relay_urls: Default::default(),
            electrum_url: "".to_string(), // Todo
            esplora_url: "".to_string(),  // Todo
        }
    }
}

impl WalletkaBuilder {
    pub fn new(
        wallet_id: Option<String>,
        database_store: DatabaseStore,
        network: Network,
        mnemonic_words: String,
        passphrase: Option<String>,
        data_path: String,
        nostr_relay_urls: Vec<String>,
        electrum_url: String,
        esplora_url: String,
    ) -> Self {
        let wallet_id = match wallet_id {
            Some(id) => id,
            None => network.to_string(),
        };

        Self {
            wallet_id: Some(wallet_id),
            database_store,
            network,
            mnemonic_words: Some(mnemonic_words),
            passphrase,
            data_path,
            nostr_relay_urls,
            electrum_url,
            esplora_url,
        }
    }

    /// Set wallet id for data store. Default is set by network
    pub fn set_wallet_id(&mut self, id: String) {
        self.wallet_id = Some(id)
    }

    pub fn set_memory_db_store(&mut self) {
        self.database_store = DatabaseStore::Memory;
    }

    pub fn set_local_db_store(&mut self, data_path: String) {
        self.database_store = DatabaseStore::Local(data_path);
    }

    pub fn set_network(&mut self, network: Network) {
        self.network = network;
    }

    pub fn set_mnemonic(&mut self, mnemonic_words: String, passphrase: Option<String>) {
        self.mnemonic_words = Some(mnemonic_words);
        self.passphrase = passphrase
    }

    pub fn set_nostr_relays(&mut self, nostr_relay_urls: Vec<String>) {
        self.nostr_relay_urls = nostr_relay_urls
    }

    pub fn add_nostr_relay(&mut self, nostr_relay_url: String) {
        self.nostr_relay_urls.push(nostr_relay_url);
    }

    pub async fn build(&self) -> Result<Walletka<Db>> {
        let database = get_database(self.database_store.clone(), Some(self.network.to_string()))
            .await
            .unwrap();
        debug!("Database created");

        let nostr_client = NostrClient::new(
            self.nostr_relay_urls.clone(),
            self.mnemonic_words.clone().unwrap(),
            self.passphrase.clone(),
        )
        .await?;
        debug!("Nostr client created");

        let contacts_repository = ContactsRepository::new(database);
        debug!("Contacts repository created");

        let contacts_service = ContactsService::new(contacts_repository, nostr_client);
        debug!("Contacts service created");

        let mnemonic = Mnemonic::parse(self.mnemonic_words.clone().unwrap())?;

        let electrum_client = bdk::electrum_client::Client::new(&self.electrum_url)?;
        let blockchain = AnyBlockchain::from(ElectrumBlockchain::from(electrum_client));
        debug!("Blockchain created");

        let bitcoin_wallet = BitcoinWallet::new(
            self.network,
            mnemonic,
            self.passphrase.clone(),
            Some(blockchain),
            self.data_path.clone(),
        )
        .unwrap();
        debug!("Bitcoin wallet created");

        let walletka = Walletka::new(contacts_service, bitcoin_wallet);
        debug!("Walletka created");

        Ok(walletka)
    }
}

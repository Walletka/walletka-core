use std::fs::{self};

use anyhow::{bail, Ok, Result};
use log::info;
use rgb_lib::wallet::{AssetNIA, DatabaseType, Online, ReceiveData, Unspent, Wallet, WalletData};
use rgb_lib::{restore_keys, BitcoinNetwork};
use tokio::task;

pub struct RgbWallet {
    inner_wallet: Wallet,
    online: Option<Online>,
    indexer_endpoint: Option<String>,
    default_transport_endpoint: Option<String>,
}

impl RgbWallet {
    pub async fn new(
        mnemonic: String,
        data_path: String,
        network: BitcoinNetwork,
        indexer_endpoint: Option<String>,
        default_transport_endpoint: Option<String>,
    ) -> Result<Self> {
        let data_path = format!("{data_path}/.rgb");

        fs::create_dir_all(data_path.clone())?;

        let keys = restore_keys(network, mnemonic)?;
        info!("RGB xpub: {}", keys.account_xpub);

        let wallet_data = WalletData {
            data_dir: data_path,
            bitcoin_network: network,
            database_type: DatabaseType::Sqlite,
            max_allocations_per_utxo: 5,
            pubkey: keys.account_xpub,
            mnemonic: Some(keys.mnemonic),
            vanilla_keychain: None,
        };

        let inner_wallet = task::block_in_place(move || -> Result<Wallet> {
            Ok(Wallet::new(wallet_data.clone())?)
        })?;

        Ok(Self {
            inner_wallet,
            online: None,
            indexer_endpoint,
            default_transport_endpoint,
        })
    }

    pub fn go_online(&mut self, endpoint: Option<String>) -> Result<()> {
        let endpoint = match endpoint {
            Some(endpoint) => endpoint,
            None => match self.indexer_endpoint.clone() {
                Some(endpoint) => endpoint,
                None => bail!("No indexer endpoint provided"),
            },
        };

        let online = self.inner_wallet.go_online(false, endpoint.clone())?;
        self.online = Some(online);
        self.indexer_endpoint = Some(endpoint);

        Ok(())
    }

    pub fn ensure_online(&mut self) -> Result<()> {
        if self.online.is_none() {
            self.go_online(None)?;
        }
        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.ensure_online()?;
        // Todo: refresh utxos and assets
        Ok(())
    }

    pub fn create_utxos(&mut self) -> Result<()> {
        self.ensure_online()?;

        self.inner_wallet
            .create_utxos(self.online.clone().unwrap(), false, None, None, 1.2)?;
        Ok(())
    }

    pub fn get_utxos(&self) -> Result<Vec<Unspent>> {
        Ok(self.inner_wallet.list_unspents(None, false)?)
    }

    pub fn issue_rgb20_asset(
        &mut self,
        ticker: String,
        name: String,
        precision: u8,
        amount: u64,
    ) -> Result<AssetNIA> {
        self.ensure_online()?;

        let asset = self.inner_wallet.issue_asset_nia(
            self.online.clone().unwrap(),
            ticker,
            name,
            precision,
            vec![amount],
        )?;

        Ok(asset)
    }

    pub fn get_rgb20_assets(&self) -> Result<Vec<AssetNIA>> {
        let assets = self.inner_wallet.list_assets(vec![])?;

        match assets.nia {
            Some(assets) => Ok(assets),
            None => Ok(vec![]),
        }
    }

    pub fn create_invoice(
        &self,
        asset_id: Option<String>,
        amount: Option<u64>,
        duration_seconds: Option<u32>,
        min_confirmations: Option<u8>,
        transport_url: Option<String>,
        blinded: bool,
    ) -> Result<ReceiveData> {
        let mut transport_endpoints = vec![];

        if let Some(transport_url) = transport_url {
            transport_endpoints.push(transport_url);
        }

        if transport_endpoints.is_empty() && self.default_transport_endpoint.is_some() {
            transport_endpoints.push(self.default_transport_endpoint.clone().unwrap());
        } else {
            bail!("No transport endpoint provided");
        }

        let min_confirmations = min_confirmations.unwrap_or(1);

        let invoice = match blinded {
            true => self.inner_wallet.blind_receive(
                asset_id,
                amount,
                duration_seconds,
                transport_endpoints,
                min_confirmations,
            )?,
            false => self.inner_wallet.witness_receive(
                asset_id,
                amount,
                duration_seconds,
                transport_endpoints,
                min_confirmations,
            )?,
        };

        Ok(invoice)
    }
}

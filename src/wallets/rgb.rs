use std::collections::HashMap;
use std::fs::{self};
use std::sync::RwLock;

use anyhow::{bail, Ok, Result};
use log::info;
use rgb_lib::wallet::{
    AssetNIA, DatabaseType, Online, ReceiveData, RefreshFilter, Unspent, Wallet, WalletData,
};
use rgb_lib::{restore_keys, BitcoinNetwork};
use tokio::task;

pub struct RgbWallet {
    inner_wallet: Wallet,
    online: Option<Online>,
    indexer_endpoint: Option<String>,
    default_transport_endpoint: Option<String>,
    assets: RwLock<HashMap<String, AssetNIA>>, // TODO: RgbAsset struct
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
            assets: RwLock::new(HashMap::new()),
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

    fn get_cached_asset(&self, asset_id: String) -> Option<AssetNIA> {
        self.assets.read().unwrap().get(&asset_id).cloned()
    }

    pub fn sync(&mut self, asset_id: Option<String>, light: bool) -> Result<bool> {
        self.ensure_online()?;

        let filter = if light {
            vec![
                RefreshFilter {
                    status: rgb_lib::wallet::RefreshTransferStatus::WaitingCounterparty,
                    incoming: true,
                },
                RefreshFilter {
                    status: rgb_lib::wallet::RefreshTransferStatus::WaitingCounterparty,
                    incoming: false,
                },
            ]
        } else {
            vec![]
        };

        let res = self
            .inner_wallet
            .refresh(self.online.clone().unwrap(), asset_id, filter)
            .unwrap();

        dbg!(&res);
        Ok(!res.is_empty()) // TODO
    }

    pub fn update_assets(
        &mut self,
        refresh: bool,
        update_transfers: bool,
        firs_refresh: bool,
    ) -> Result<()> {
        if refresh && !firs_refresh {
            self.sync(None, false)?;
        }

        let assets = self.get_rgb20_assets()?;

        for asset in assets {
            let mut next_update_transfers = update_transfers;
            let mut asset_to_update = self.get_cached_asset(asset.asset_id.clone());

            if asset_to_update.is_none() {
                asset_to_update = Some(asset.clone());
                next_update_transfers = true;
                self.assets
                    .write()
                    .unwrap()
                    .insert(asset.asset_id.clone(), asset.clone());
            } else {
                let mut asset_to_update_mut = asset_to_update.unwrap();
                asset_to_update_mut.balance.spendable = asset.balance.spendable;
                asset_to_update_mut.balance.settled = asset.balance.settled;
                asset_to_update_mut.balance.future = asset.balance.future;

                asset_to_update = Some(asset_to_update_mut);
            }

            self.update_asset(asset.asset_id, firs_refresh, next_update_transfers, None)?;
        }

        // Todo first app refresh

        Ok(())
    }

    pub fn update_asset(
        &mut self,
        asset_id: String,
        refresh: bool,
        update_transfers: bool,
        _update_transfers_filter: Option<String>,
    ) -> Result<()> {
        let mut call_list_transfers = update_transfers;
        if refresh {
            call_list_transfers = self.sync(Some(asset_id.clone()), false)? || call_list_transfers;
            // Todo update balance
        }

        if call_list_transfers {
            // Todo list transfers
        }

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
        } 
        if transport_endpoints.is_empty() {
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

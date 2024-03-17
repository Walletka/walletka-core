use itertools::Itertools;

use anyhow::Result;
use bdk::bitcoin::{psbt::PartiallySignedTransaction, Address, Transaction};
use surrealdb::Connection;

use crate::{
    enums::WalletkaAssetState,
    io::entities::WalletkaContact,
    services::ContactsService,
    types::{Amount, WalletkaAsset, WalletkaBalance},
    wallets::bitcoin::BitcoinWallet,
};

pub struct Walletka<C>
where
    C: Connection,
{
    contact_service: ContactsService<C>,
    bitcoin_wallet: BitcoinWallet,
}

impl<C> Walletka<C>
where
    C: Connection,
{
    pub fn new(contact_service: ContactsService<C>, bitcoin_wallet: BitcoinWallet) -> Self {
        Self {
            contact_service,
            bitcoin_wallet,
        }
    }

    pub async fn add_contact(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        self.contact_service.add(contact).await
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<WalletkaContact>> {
        self.contact_service.get_all().await
    }

    pub async fn delete_contact(&self, contact: WalletkaContact) -> Result<()> {
        self.contact_service.delete(contact).await
    }

    pub async fn update_contact(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        self.contact_service.update(contact).await
    }

    pub async fn import_contacts_from_npub(&self, npub: String) -> Result<Vec<String>> {
        self.contact_service.import_from_npub(npub).await
    }

    /// Sync wallets
    pub async fn sync(&self) -> Result<()> {
        self.bitcoin_wallet.sync()
    }

    pub fn sign_psbt(&self, psbt: &mut PartiallySignedTransaction) -> Result<()> {
        self.bitcoin_wallet.sign_psbt(psbt)
    }

    /// Broadcast transaction to the mempool
    pub fn broadcast_tx(&self, transaction: &Transaction) -> Result<()> {
        self.bitcoin_wallet.broadcast_tx(&transaction)
    }

    /// Get all assets held by Walletka
    pub fn get_assets(&self) -> Result<Vec<WalletkaAsset>> {
        let utxos: Vec<WalletkaAsset> = self
            .bitcoin_wallet
            .get_utxos()?
            .into_iter()
            .map(WalletkaAsset::from)
            .collect();

        Ok(utxos)
    }

    /// Get onchain address
    pub fn get_bitcoin_address(&self) -> Result<Address> {
        self.bitcoin_wallet.get_unused_address()
    }

    /// Get all assets grouped by currency
    pub fn get_balance(&self, currency_symbol: Option<String>) -> Result<WalletkaBalance> {
        let mut walletka_balance = WalletkaBalance::default();

        let mut assets = self.get_assets()?;

        if let Some(symbol) = currency_symbol {
            assets = assets
                .into_iter()
                .filter(|a| a.amount.currency.symbol == symbol)
                .collect();
        }

        let by_currency = assets.into_iter().group_by(|a| a.amount.currency.clone());

        for assets in by_currency.into_iter() {
            let mut confirmed_value = 0;
            let mut unconfirmed_value = 0;

            for asset in assets.1 {
                if asset.asset_state == WalletkaAssetState::Settled {
                    confirmed_value += asset.amount.value;
                } else {
                    unconfirmed_value += asset.amount.value;
                }
            }

            if confirmed_value > 0 {
                let confirmed_amount = Amount {
                    currency: assets.0.clone(),
                    value: confirmed_value,
                };
                walletka_balance.confirmed.push(confirmed_amount);
            }

            if unconfirmed_value > 0 {
                let unconfirmed_amount = Amount {
                    currency: assets.0,
                    value: unconfirmed_value,
                };

                walletka_balance.unconfirmed.push(unconfirmed_amount);
            }
        }

        Ok(walletka_balance)
    }
}

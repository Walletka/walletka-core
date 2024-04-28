use itertools::Itertools;

use anyhow::Result;
use bdk::bitcoin::{psbt::PartiallySignedTransaction, Address, Transaction};
use surrealdb::Connection;

use crate::{
    enums::WalletkaAssetState,
    io::entities::{CashuMint, WalletkaContact},
    services::ContactsManager,
    types::{Amount, WalletkaAsset, WalletkaBalance},
    wallets::{bitcoin::BitcoinWallet, cashu::CashuWallet},
};

pub struct Walletka<C>
where
    C: Connection,
{
    contact_manager: ContactsManager<C>,
    bitcoin_wallet: BitcoinWallet,
    cashu_wallet: CashuWallet<C>,
}

impl<C> Walletka<C>
where
    C: Connection,
{
    pub fn new(
        contact_service: ContactsManager<C>,
        bitcoin_wallet: BitcoinWallet,
        cashu_wallet: CashuWallet<C>,
    ) -> Self {
        Self {
            contact_manager: contact_service,
            bitcoin_wallet,
            cashu_wallet,
        }
    }

    pub async fn add_contact(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        self.contact_manager.add(contact).await
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<WalletkaContact>> {
        self.contact_manager.get_all().await
    }

    pub async fn delete_contact_by_id(&self, contact_id: String) -> Result<()> {
        let contact = self.contact_manager.get_by_id(&contact_id).await?;

        self.contact_manager.delete(contact).await
    }

    pub async fn update_contact(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        self.contact_manager.update(contact).await
    }

    pub async fn import_contacts_from_npub(&self, npub: String) -> Result<Vec<String>> {
        self.contact_manager.import_from_npub(npub).await
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
    pub async fn get_assets(&self) -> Result<Vec<WalletkaAsset>> {
        let mut walletka_assets: Vec<WalletkaAsset> = vec![];

        let mut utxos: Vec<WalletkaAsset> = self
            .bitcoin_wallet
            .get_utxos()?
            .into_iter()
            .map(WalletkaAsset::from)
            .collect();

        let mut cashu_tokens: Vec<WalletkaAsset> = self
            .cashu_wallet
            .get_proofs()
            .await?
            .into_iter()
            .map(WalletkaAsset::from)
            .collect();

        let mut cashu_pending_tokens: Vec<WalletkaAsset> = self
            .cashu_wallet
            .get_pending_tokens()
            .await?
            .into_iter()
            .map(WalletkaAsset::from)
            .collect();

        walletka_assets.append(&mut utxos);
        walletka_assets.append(&mut cashu_tokens);
        walletka_assets.append(&mut cashu_pending_tokens);

        Ok(walletka_assets)
    }

    /// Get onchain address
    pub fn get_bitcoin_address(&self) -> Result<Address> {
        self.bitcoin_wallet.get_unused_address()
    }

    /// Get all assets grouped by currency
    pub async fn get_balance(&self, currency_symbol: Option<String>) -> Result<WalletkaBalance> {
        let mut walletka_balance = WalletkaBalance::default();

        let mut assets = self.get_assets().await?;

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

    pub async fn claim_cashu_token(&mut self, token: String) -> Result<()> {
        self.cashu_wallet.claim_token(token).await
    }

    pub async fn get_cashu_mints(&self) -> Result<Vec<CashuMint>> {
        Ok(self.cashu_wallet.get_mints())
    }

    pub async fn send_cashu_token(
        &mut self,
        keyset_id: String,
        amount_sat: u64,
        memo: Option<String>,
    ) -> Result<String> {
        let token = self
            .cashu_wallet
            .create_token_from_keyset(keyset_id, amount_sat, memo)
            .await?;

        Ok(token.convert_to_string()?)
    }
}

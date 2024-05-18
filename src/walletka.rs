use itertools::Itertools;

use anyhow::Result;
use bdk::bitcoin::{psbt::PartiallySignedTransaction, Address, Transaction};
use log::info;

use crate::{
    enums::WalletkaAssetState,
    io::entities::CashuMint,
    types::{Amount, WalletkaAsset, WalletkaBalance},
    wallets::{bitcoin::BitcoinWallet, cashu::CashuWallet, rgb::RgbWallet, NestedWallet},
};

pub struct Walletka
{
    bitcoin_wallet: BitcoinWallet,
    cashu_wallet: CashuWallet,
    rgb_wallet: RgbWallet,
}

impl Walletka
{
    pub fn new(
        bitcoin_wallet: BitcoinWallet,
        cashu_wallet: CashuWallet,
        rgb_wallet: RgbWallet,
    ) -> Self {
        Self {
            bitcoin_wallet,
            cashu_wallet,
            rgb_wallet,
        }
    }

    /// Sync wallets
    pub async fn sync(&mut self) -> Result<()> {
        // Todo: Parallelize
        self.bitcoin_wallet.sync()?;
        self.rgb_wallet.sync()?;

        Ok(())
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

        let mut rgb_utxos: Vec<WalletkaAsset> = self
            .rgb_wallet
            .get_utxos()?
            .into_iter()
            .filter(|u| u.utxo.colorable)
            .map(WalletkaAsset::from)
            .collect();

        let mut rgb_assets: Vec<WalletkaAsset> = self
            .rgb_wallet
            .get_rgb20_assets()?
            .into_iter()
            .map(WalletkaAsset::from)
            .collect();

        walletka_assets.append(&mut utxos);
        walletka_assets.append(&mut cashu_tokens);
        walletka_assets.append(&mut cashu_pending_tokens);
        walletka_assets.append(&mut rgb_utxos);
        walletka_assets.append(&mut rgb_assets);

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
            let mut locked_value = 0;

            for asset in assets.1 {
                if asset.asset_state == WalletkaAssetState::Settled {
                    confirmed_value += asset.amount.value;
                } else if asset.asset_state == WalletkaAssetState::Unspendable {
                    // Todo: Add pending state
                    locked_value += asset.amount.value;
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
                    currency: assets.0.clone(),
                    value: unconfirmed_value,
                };

                walletka_balance.unconfirmed.push(unconfirmed_amount);
            }

            if locked_value > 0 {
                let locked_amount = Amount {
                    currency: assets.0,
                    value: locked_value,
                };

                walletka_balance.locked.push(locked_amount);
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

    // RGB functions

    pub fn create_rgb_utxos(&mut self) -> Result<()> {
        self.rgb_wallet.create_utxos()
    }

    pub fn issue_rgb20_asset(
        &mut self,
        ticker: String,
        name: String,
        precision: u8,
        amount: u64,
    ) -> Result<String> {
        info!("Issuing RGB20 asset");

        let asset = self.rgb_wallet
            .issue_rgb20_asset(ticker, name, precision, amount)?;

        info!("RGB20 asset issued: {}", asset.asset_id);
        Ok(asset.asset_id)
    }

    pub fn create_rgb_invoice(
        &self,
        asset_id: Option<String>,
        amount: Option<u64>,
        duration_seconds: Option<u32>,
        min_confirmations: Option<u8>,
        transport_url: Option<String>,
        blinded: bool,
    ) -> Result<String> {
        info!("Creating RGB invoice");

        let invoice_data = self.rgb_wallet.create_invoice(
            asset_id,
            amount,
            duration_seconds,
            min_confirmations,
            transport_url,
            blinded,
        )?;

        info!("RGB invoice created: {}", invoice_data.invoice);

        Ok(invoice_data.invoice)
    }
}

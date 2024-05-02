use bdk::LocalUtxo;
use rgb_lib::wallet::{AssetNIA, Unspent};
use serde::{Deserialize, Serialize};

use crate::{
    enums::{WalletkaAssetLocation, WalletkaAssetState, WalletkaLayer},
    io::entities::{CashuProof, PendingCashuToken},
};

use super::{Amount, Currency};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct WalletkaAsset {
    pub layer: WalletkaLayer,
    pub asset_location: WalletkaAssetLocation,
    pub asset_state: WalletkaAssetState,
    pub amount: Amount,
}

impl From<LocalUtxo> for WalletkaAsset {
    fn from(value: LocalUtxo) -> Self {
        Self {
            layer: WalletkaLayer::Blockchain,
            asset_location: WalletkaAssetLocation::Utxo(format!(
                "{}:{}",
                value.outpoint.txid, value.outpoint.vout
            )),
            asset_state: if value.is_spent {
                WalletkaAssetState::Spent
            } else {
                WalletkaAssetState::Settled
            },
            amount: Amount::new(value.txout.value, Currency::bitcoin()),
        }
    }
}

impl From<CashuProof> for WalletkaAsset {
    fn from(value: CashuProof) -> Self {
        Self {
            layer: WalletkaLayer::Cashu,
            asset_location: WalletkaAssetLocation::Cashu(value.id.unwrap().id.to_string()),
            asset_state: WalletkaAssetState::Settled,
            amount: Amount::new(value.amount_sat, Currency::bitcoin()),
        }
    }
}

impl From<PendingCashuToken> for WalletkaAsset {
    fn from(value: PendingCashuToken) -> Self {
        Self {
            layer: WalletkaLayer::Cashu,
            asset_location: WalletkaAssetLocation::Cashu(value.id.clone().unwrap().id.to_string()),
            asset_state: WalletkaAssetState::Waiting,
            amount: Amount::new(value.amount_sat, Currency::bitcoin()),
        }
    }
}

impl From<Unspent> for WalletkaAsset {
    fn from(value: Unspent) -> Self {
        Self {
            layer: WalletkaLayer::Rgb,
            asset_location: WalletkaAssetLocation::Utxo(format!(
                "{}:{}",
                value.utxo.outpoint.txid, value.utxo.outpoint.vout
            )),
            asset_state: WalletkaAssetState::Unspendable,
            amount: Amount::new(value.utxo.btc_amount, Currency::bitcoin()),
        }
    }
}

impl From<AssetNIA> for WalletkaAsset {
    fn from(value: AssetNIA) -> Self {
        Self {
            layer: WalletkaLayer::Rgb,
            asset_location: WalletkaAssetLocation::Utxo(format!(
                "{}",
                value.asset_id
            )),
            asset_state: WalletkaAssetState::Unknown,
            amount: Amount::new(value.balance.settled, Currency::new(
                value.ticker.clone(),
                value.name,
                value.ticker,
                value.precision as u64,
            )),
        }
    }
}

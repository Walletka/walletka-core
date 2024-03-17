use bdk::LocalUtxo;
use serde::{Deserialize, Serialize};

use crate::enums::{WalletkaAssetLocation, WalletkaAssetState, WalletkaLayer};

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

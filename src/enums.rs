use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionDirection {
    Received,
    Sent,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum WalletkaAssetLocation {
    Utxo(String),
    LightningChannel(String),
    Cashu(String),
    Fedimint(String),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WalletkaLayer {
    Blockchain,
    Lightning,
    Cashu,
    Fedimint,
    Rgb,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum WalletkaAssetState {
    Unknown,
    Waiting,
    Settled,
    Spent,
}

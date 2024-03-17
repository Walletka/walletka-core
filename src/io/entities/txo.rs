use bdk::LocalUtxo;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Txo {
    pub id: Option<Thing>,
    pub tx_id: String,
    pub vout: u32,
    pub amount_sats: u64,
    pub spent: bool,
}

impl From<LocalUtxo> for Txo {
    fn from(value: LocalUtxo) -> Self {
        Self {
            id: None,
            tx_id: value.outpoint.txid.to_string(),
            vout: value.outpoint.vout,
            amount_sats: value.txout.value,
            spent: false,
        }
    }
}

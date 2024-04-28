use std::str::FromStr;

use cashu_sdk::{
    nuts::{Id, Proof, PublicKey},
    secret::Secret,
    Amount,
};
use nostr_sdk::serde_json::value;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CashuProof {
    pub id: Option<Thing>,
    pub keyset_id: Option<Id>,
    pub amount_sat: u64,
    pub secret: Secret,
    pub c: PublicKey,
}

impl CashuProof {
    pub fn new(keyset_id: Option<Id>, amount_sat: u64, secret: Secret, c: PublicKey) -> Self {
        Self {
            id: None,
            keyset_id,
            amount_sat,
            secret,
            c,
        }
    }
}

impl From<&Proof> for CashuProof {
    fn from(value: &Proof) -> Self {
        Self {
            id: None,
            keyset_id: value.id,
            amount_sat: value.amount.to_sat(),
            secret: value.secret.clone(),
            c: value.c.clone(),
        }
    }
}

impl Into<Proof> for &CashuProof {
    fn into(self) -> Proof {
        Proof {
            amount: Amount::from_sat(self.amount_sat),
            secret: self.secret.clone(),
            c: self.c.clone(),
            id: self.keyset_id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PendingCashuToken {
    pub id: Option<Thing>,
    pub claimed: bool,
    pub datetime: Datetime,
    pub token: String,
    pub source: PendingCashuTokenSource,
    pub amount_sat: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PendingCashuTokenSource {
    Received,
    Sent,
}

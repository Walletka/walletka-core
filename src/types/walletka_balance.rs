use serde::{Deserialize, Serialize};

use super::Amount;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WalletkaBalance {
    pub confirmed: Vec<Amount>,
    pub unconfirmed: Vec<Amount>,
}

impl WalletkaBalance {}

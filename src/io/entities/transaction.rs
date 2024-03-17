use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::enums::TransactionDirection;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletkaTransaction {
    pub id: Option<Thing>,
    pub direction: TransactionDirection,
}

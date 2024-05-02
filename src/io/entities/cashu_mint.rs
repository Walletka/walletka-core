use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CashuMint {
    pub mint_url: String,
    pub trust_level: i32,
    pub keyset_id: String,
}

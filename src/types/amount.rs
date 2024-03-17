use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Currency {
    pub symbol: String,
    pub name: String,
    pub base_unit_symbol: String,
    pub decimals: u64,
}

impl Currency {
    pub fn new(symbol: String, name: String, base_unit_symbol: String, decimals: u64) -> Self {
        Self {
            symbol,
            name,
            base_unit_symbol,
            decimals,
        }
    }

    pub fn bitcoin() -> Self {
        Self {
            symbol: "Btc".to_string(),
            name: "Bitcoin".to_string(),
            base_unit_symbol: "sat".to_string(),
            decimals: 8,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Amount {
    pub value: u64,
    pub currency: Currency,
}

impl Amount {
    pub fn new(value: u64, currency: Currency) -> Self {
        Self { value, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self { value: 0, currency }
    }

    pub fn base_value(&self) -> u64 {
        self.value * self.currency.decimals
    }
}

use std::str::FromStr;

use walletka_core::bdk::{keys::{bip39::{Language, Mnemonic, WordCount}, GeneratableKey, GeneratedKey}, miniscript};



pub fn generate_mnemonic() -> String {
    let op: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();

    op.to_string()
}

pub fn validate_mnemonic(mnemonic: String) -> bool {
    Mnemonic::from_str(&mnemonic).is_ok()
}
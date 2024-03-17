pub use bdk;
pub use walletka::Walletka;

pub mod builder;
pub mod commands;
pub mod enums;
pub mod errors;
pub mod io;
pub mod services;
pub mod types;
pub mod utils;
pub mod walletka;
pub mod wallets;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bdk::{
        keys::{
            bip39::{Language, Mnemonic, WordCount},
            GeneratableKey, GeneratedKey,
        },
        miniscript,
    };

    use super::*;

    #[test]
    fn it_works() {
        let data_path = ".test_data";

        if Path::new(data_path).exists() {
            std::fs::remove_dir_all(data_path).unwrap();
        }

        let op: GeneratedKey<_, miniscript::Segwitv0> =
            Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
        let mnemonic = Mnemonic::parse(op.to_string()).unwrap();

        let bitcoin_wallet = wallets::bitcoin::BitcoinWallet::new(
            bdk::bitcoin::Network::Regtest,
            mnemonic,
            None,
            None,
            data_path.to_string(),
        )
        .expect("Can't create bitcoin wallet");

        bitcoin_wallet
            .get_unused_address()
            .expect("Can't get address");
    }
}

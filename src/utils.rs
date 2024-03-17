use std::fs;

use bdk::{
    keys::{
        bip39::{Language, Mnemonic, WordCount},
        GeneratableKey, GeneratedKey,
    },
    miniscript,
};
use log::{debug, error};

pub fn generate_mnemonic() -> String {
    debug!("Generating mnemonic...");
    let op: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();

    debug!("Mnemonic generated");
    op.to_string()
}

pub fn save_mnemonic(mnemonic: &str, path: &str) -> Result<(), anyhow::Error> {
    let walet_file = format!("{}/wallet.dat", path);
    debug!("Saving mnemonic to {}", walet_file);

    if fs::File::open(walet_file.clone()).is_err() {
        fs::File::create(walet_file.clone()).expect("Cannot create wallet file");
    }

    match fs::write(walet_file, mnemonic) {
        Ok(()) => {
            debug!("Mnemonic saved to file!");
            Ok(())
        }
        Err(e) => {
            error!("Error saving mnemonic");
            Err(e.into())
        }
    }
}

pub fn load_mnemonic(path: &str) -> Result<Mnemonic, anyhow::Error> {
    let walet_file = format!("{}/wallet.dat", path);
    match fs::read_to_string(walet_file) {
        Ok(mnemonic_words) => Ok(Mnemonic::parse(&mnemonic_words)?),
        Err(e) => Err(e.into()),
    }
}

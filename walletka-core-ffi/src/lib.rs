use std::sync::Mutex;

use walletka_core::builder::WalletkaBuilder as SdkBuilder;

uniffi::include_scaffolding!("walletka-core");

pub struct WalletkaBuilder {
    inner_builder: Mutex<SdkBuilder>,
}

impl WalletkaBuilder {
    pub fn new() -> Self {
        Self {
            inner_builder: Mutex::new(SdkBuilder::default()),
        }
    }

    pub fn set_mnemonic(&self, mnemonic_words: String) {
        self.inner_builder
            .lock()
            .unwrap()
            .set_mnemonic(mnemonic_words, None);
    }
}

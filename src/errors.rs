use bdk::electrum_client;

#[derive(Debug, thiserror::Error)]
pub enum WalletkaError {
    /// An invalid bitcoin address has been provided
    #[error("Address error: {details}")]
    InvalidAddress {
        /// Error details
        details: String,
    },

    /// The provided mnemonic phrase is invalid
    #[error("Invalid mnemonic error: {details}")]
    InvalidMnemonic {
        /// Error details
        details: String,
    },

    /// The provided PSBT could not be parsed
    #[error("Invalid PSBT: {details}")]
    InvalidPsbt {
        /// Error details
        details: String,
    },

    /// The provided pubkey is invalid
    #[error("Invalid pubkey: {details}")]
    InvalidPubkey {
        /// Error details
        details: String,
    },

    /// The provided script is invalid
    #[error("Invalid script: {details}")]
    InvalidScript {
        /// Error details
        details: String,
    },

    /// An error was received from the Electrum server
    #[error("Electrum error: {details}")]
    Electrum {
        /// Error details
        details: String,
    },

    /// An error in bdk
    #[error("Bdk error: {details}")]
    Bdk {
        /// Error details
        details: String,
    },
}

impl From<bdk::keys::bip39::Error> for WalletkaError {
    fn from(e: bdk::keys::bip39::Error) -> Self {
        WalletkaError::InvalidMnemonic {
            details: e.to_string(),
        }
    }
}

impl From<bdk::bitcoin::address::Error> for WalletkaError {
    fn from(e: bdk::bitcoin::address::Error) -> Self {
        WalletkaError::InvalidAddress {
            details: e.to_string(),
        }
    }
}

impl From<bdk::Error> for WalletkaError {
    fn from(e: bdk::Error) -> Self {
        WalletkaError::Bdk {
            details: e.to_string(),
        }
    }
}

impl From<bdk::bitcoin::bip32::Error> for WalletkaError {
    fn from(e: bdk::bitcoin::bip32::Error) -> Self {
        WalletkaError::InvalidPubkey {
            details: e.to_string(),
        }
    }
}

impl From<bdk::bitcoin::psbt::PsbtParseError> for WalletkaError {
    fn from(e: bdk::bitcoin::psbt::PsbtParseError) -> Self {
        WalletkaError::InvalidPsbt {
            details: e.to_string(),
        }
    }
}

impl From<electrum_client::Error> for WalletkaError {
    fn from(e: electrum_client::Error) -> Self {
        WalletkaError::Electrum {
            details: e.to_string(),
        }
    }
}

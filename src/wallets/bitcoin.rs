use anyhow::{bail, Result};
use bdk::bitcoin::bip32::ExtendedPubKey;
use bdk::bitcoin::psbt::PartiallySignedTransaction;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::{Address, Network, Transaction};
use bdk::blockchain::{AnyBlockchain, Blockchain, ElectrumBlockchain};
use bdk::database::any::SledDbConfiguration;
use bdk::database::{AnyDatabase, ConfigurableDatabase};
use bdk::electrum_client::Client;
use bdk::keys::{bip39::Mnemonic, DerivableKey, ExtendedKey};
use bdk::template::Bip84;
use bdk::wallet::AddressIndex;
use bdk::{Balance, LocalUtxo, SignOptions, SyncOptions, Wallet as BdkWallet};
use log::{debug, info};
use crate::wallets::NestedWallet;

pub struct BitcoinWallet {
    wallet: BdkWallet<AnyDatabase>,
    pub xpub: ExtendedPubKey,
    blockchain: Option<AnyBlockchain>,
}

impl NestedWallet for BitcoinWallet {

 fn sync(&self) -> Result<()> {
        debug!("Syncing with blockchain...");

        match &self.blockchain {
            Some(blockchain) => {
                self.wallet.sync(blockchain, SyncOptions::default())?;
                info!("Blockchain synced");
                Ok(())
            }
            None => bail!("Offline mode"),
        }
    }
}

impl BitcoinWallet {
    pub fn new_with_electrum(
        network: Network,
        mnemonic: Mnemonic,
        passphrase: Option<String>,
        electrum_url: String,
        data_path: String,
    ) -> Result<BitcoinWallet, anyhow::Error> {
        let client = Client::new(&electrum_url)?;
        let blockchain = AnyBlockchain::from(ElectrumBlockchain::from(client));

        let data_path = format!("{data_path}/bdk");

        BitcoinWallet::new(network, mnemonic, passphrase, Some(blockchain), data_path)
    }

    pub fn new(
        network: Network,
        mnemonic: Mnemonic,
        _passphrase: Option<String>,
        blockchain: Option<AnyBlockchain>,
        data_path: String,
    ) -> Result<BitcoinWallet> {
        let data_path = format!("{data_path}/.bdk");

        let secp = Secp256k1::new();

        let bdk_config = SledDbConfiguration {
            path: data_path,
            tree_name: "MAIN_WALLET".to_string(),
        };
        let database = AnyDatabase::from_config(&bdk_config.into())?;

        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.clone().into_extended_key()?;
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).unwrap();
        let xpub = ExtendedPubKey::from_priv(&secp, &xprv);

        info!("Xpub:\n{}", xpub);

        let descriptor = Bip84(xprv, bdk::KeychainKind::Internal);
        let change_descriptor = Some(Bip84(xprv, bdk::KeychainKind::Internal));

        // Todo: Create a BDK wallet structure using BIP 86 descriptor ("m/86'/0'/0'/0" and "m/86'/0'/0'/1")
        let wallet = BdkWallet::new(
            descriptor,
            change_descriptor,
            network,
            database,
        )?;

        Ok(BitcoinWallet {
            wallet,
            xpub,
            blockchain,
        })
    }


    pub fn get_unused_address(&self) -> Result<Address> {
        Ok(self.wallet.get_address(AddressIndex::LastUnused)?.address)
    }

    pub fn get_balance(&self) -> Result<Balance> {
        Ok(self.wallet.get_balance()?)
    }

    pub fn get_utxos(&self) -> Result<Vec<LocalUtxo>> {
        Ok(self.wallet.list_unspent()?)
    }

    pub fn pay_to_address(&self, address: Address, amount_sat: u64, rbf: bool) -> Result<String> {
        let mut builder = self.wallet.build_tx();

        if rbf {
            builder.enable_rbf();
        }
        builder.add_recipient(address.script_pubkey(), amount_sat);
        // Todo

        let mut psbt = builder.finish()?.0;
        self.sign_psbt(&mut psbt)?;

        let tx = psbt.extract_tx();

        match self.broadcast_tx(&tx) {
            Ok(_) => Ok(tx.txid().to_string()),
            Err(err) => Err(err),
        }
    }

    pub fn sign_psbt(&self, psbt: &mut PartiallySignedTransaction) -> Result<()> {
        self.wallet.sign(psbt, SignOptions::default())?;
        Ok(())
    }

    pub fn broadcast_tx(&self, transaction: &Transaction) -> Result<()> {
        match &self.blockchain {
            Some(blockchain) => {
                let tx = transaction;
                blockchain.broadcast(tx)?;
                Ok(())
            }
            None => bail!("Offline mode!"),
        }
    }
}

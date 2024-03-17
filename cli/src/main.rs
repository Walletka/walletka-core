use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::{debug, info};
use walletka_core::{
    bdk::bitcoin::{bech32::ToBase32, Network},
    builder::WalletkaBuilder,
    io::database::DatabaseStore,
    utils::{generate_mnemonic, load_mnemonic, save_mnemonic},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Commands,

    #[arg(global = true)]
    file: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Info,
    Sync,
    Address,
    Assets,
    Balance { currency_symbol: Option<String> },
    Contacts,
    ImportContacts { npub: String },
    DeleteContact { contact_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let data_path = ".data".to_string();
    let nostr_relay_url = "wss://nostr.tchaicap.space".to_string();

    if fs::File::open(data_path.clone()).is_err() {
        fs::create_dir(data_path.clone())?;
    }

    let mnemonic_words = match load_mnemonic(&data_path) {
        Ok(mnemonic) => mnemonic.to_string(),
        Err(_) => {
            let mnemonic = generate_mnemonic();
            save_mnemonic(&mnemonic, &data_path)?;
            mnemonic
        }
    };

    let args = Args::parse();

    debug!("Creating Walletka builder");
    let builder = WalletkaBuilder::new(
        None,
        DatabaseStore::Local(data_path.clone()),
        Network::Regtest,
        mnemonic_words,
        None,
        data_path,
        vec![nostr_relay_url],
        "100.64.147.89:50000".to_string(),
        "esplora.tchaicash.space:443".to_string(),
    );

    debug!("Building Walletka...");
    let walletka = builder.build().await?;
    info!("Walletka started");

    match args.cmd {
        Commands::Info => todo!(),
        Commands::Sync => {
            debug!("Syncing Walletka");
            walletka.sync().await?;
        }
        Commands::Address => {
            let address = walletka.get_bitcoin_address()?;
            dbg!(address);
        }
        Commands::Assets => {
            let assets = walletka.get_assets()?;
            dbg!(assets);
        }
        Commands::Balance { currency_symbol } => {
            let balance = walletka.get_balance(currency_symbol)?;
            info!("Balance: {:#? }", balance);
        }
        Commands::Contacts => {
            let contacts = walletka.get_all_contacts().await?;
            dbg!(contacts);
        }
        Commands::ImportContacts { npub } => {
            walletka.import_contacts_from_npub(npub).await?;
        }
        Commands::DeleteContact { contact_id } => {
            walletka.delete_contact_by_id(contact_id).await?;
        }
    };

    Ok(())
}

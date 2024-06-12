use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::{debug, info};
use walletka_core::{
    bdk::bitcoin::Network,
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
    Sync {
        light: Option<bool>,
    },
    Address,
    Assets,
    Balance {
        currency_symbol: Option<String>,
    },
    Contacts,
    ImportContacts {
        npub: String,
    },
    DeleteContact {
        contact_id: String,
    },
    CashuClaim {
        token: String,
    },
    CashuMints,
    CashuSend {
        keyset_id: String,
        amount_sat: u64,
    },
    RgbCreateUtxos,
    RgbCreateAssetNia {
        ticker: String,
        name: String,
        precision: u8,
        amount: u64,
    },
    RgbInvoice {
        asset_id: Option<String>,
        amount: Option<u64>,
        duration_seconds: Option<u32>,
        min_confirmations: Option<u8>,
        transport_url: Option<String>,
        blinded: Option<bool>,
    },
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
        Some("130.61.74.161:50001".to_string()),
        Some("esplora.tchaicash.space:443".to_string()),
        Some("rpc://rgb.tchaicash.space:443".to_string()), // Todo: Some("rgb.tchaicash.space:443".to_string()),
    );

    debug!("Building Walletka...");
    let mut walletka = builder.build().await?;
    info!("Walletka started");

    match args.cmd {
        Commands::Info => todo!(),
        Commands::Sync { light } => {
            debug!("Syncing Walletka");
            walletka.sync(light.unwrap_or(false)).await?;
        }
        Commands::Address => {
            let address = walletka.get_bitcoin_address()?;
            dbg!(address);
        }
        Commands::Assets => {
            let assets = walletka.get_assets().await?;
            dbg!(assets);
        }
        Commands::Balance { currency_symbol } => {
            let balance = walletka.get_balance(currency_symbol).await?;
            info!("Balance: {:#? }", balance);
        }
        Commands::Contacts => {
            // let contacts = walletka.get_all_contacts().await?;
            // dbg!(contacts);
            println!("TODO");
        }
        Commands::ImportContacts { npub: _ } => {
            // walletka.import_contacts_from_npub(npub).await?;
            println!("TODO");
        }
        Commands::DeleteContact { contact_id: _ } => {
            // walletka.delete_contact_by_id(contact_id).await?;
            println!("TODO");
        }
        Commands::CashuClaim { token } => {
            walletka.claim_cashu_token(token).await?;
        }
        Commands::CashuMints => {
            let mints = walletka.get_cashu_mints().await?;
            dbg!(mints);
        }
        Commands::CashuSend {
            keyset_id,
            amount_sat,
        } => {
            let token = walletka
                .send_cashu_token(
                    keyset_id,
                    amount_sat,
                    Some("Send from walletka".to_string()),
                )
                .await?;
            dbg!(token);
        }
        Commands::RgbCreateUtxos => {
            walletka.create_rgb_utxos()?;
            info!("Utxos created");
        }
        Commands::RgbCreateAssetNia {
            ticker,
            name,
            precision,
            amount,
        } => {
            let asset_id = walletka.issue_rgb20_asset(ticker, name, precision, amount)?;
            info!("Asset created: {}", asset_id);
        }

        Commands::RgbInvoice {
            asset_id,
            amount,
            duration_seconds,
            min_confirmations,
            transport_url,
            blinded,
        } => {
            let invoice = walletka.create_rgb_invoice(
                asset_id,
                amount,
                duration_seconds,
                min_confirmations,
                transport_url,
                blinded.unwrap_or(true),
            )?;
            dbg!(invoice);
        }
    };

    Ok(())
}

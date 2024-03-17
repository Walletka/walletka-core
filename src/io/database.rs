use anyhow::{Ok, Result};
use bdk::bitcoin::Network;
use surrealdb::{
    engine::local::{Db, Mem, RocksDb},
    Surreal,
};

#[derive(Clone)]
pub enum DatabaseStore {
    Local(String),
    Memory,
    Remote(String),
}

pub async fn get_database(store: DatabaseStore, ns: Option<String>) -> Result<Surreal<Db>> {
    let ns = match ns {
        Some(name) => name,
        None => Network::Bitcoin.to_string(),
    };

    match store {
        DatabaseStore::Local(path) => {
            let db = Surreal::new::<RocksDb>(format!("{}/.db", path)).await?;
            db.use_ns(ns).use_db("core").await?;
            Ok(db)
        }
        DatabaseStore::Memory => Ok(Surreal::new::<Mem>(()).await?),
        DatabaseStore::Remote(_url) => todo!(),
    }
}

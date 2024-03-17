use anyhow::Result;
use surrealdb::{Connection, Surreal};

use crate::io::entities::Txo;

pub struct BitcoinRepository<C>
where
    C: Connection,
{
    database: Surreal<C>,
}

impl<C> BitcoinRepository<C>
where
    C: Connection,
{
    pub fn new(database: Surreal<C>) -> Self {
        Self { database }
    }
}

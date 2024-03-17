use surrealdb::{Connection, Surreal};

pub struct TransactionRepository<C>
where
    C: Connection,
{
    database: Surreal<C>,
}

impl<C> TransactionRepository<C>
where
    C: Connection,
{
    pub fn new(database: Surreal<C>) -> Self {
        Self { database }
    }

    
}

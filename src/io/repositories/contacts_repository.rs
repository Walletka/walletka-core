use anyhow::{bail, Result};
use surrealdb::{Connection, Surreal};

use crate::io::entities::WalletkaContact;

const TABLE_NAME: &str = "contacts";

pub struct ContactsRepository<C>
where
    C: Connection,
{
    database: Surreal<C>,
}

impl<C> ContactsRepository<C>
where
    C: Connection,
{
    pub fn new(database: Surreal<C>) -> Self {
        Self { database }
    }

    pub async fn add(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        let created: Vec<WalletkaContact> =
            self.database.create(TABLE_NAME).content(contact).await?;

        Ok(created.into_iter().last().unwrap())
    }

    pub async fn get_all(&self) -> Result<Vec<WalletkaContact>> {
        let contacts: Vec<WalletkaContact> = self.database.select(TABLE_NAME).await?;

        Ok(contacts)
    }

    pub async fn delete(&self, contact: WalletkaContact) -> Result<()> {
        let _: Option<WalletkaContact> = self
            .database
            .delete((TABLE_NAME, contact.id.unwrap().id))
            .await?;
        Ok(())
    }

    pub async fn update(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        let id = match contact.id.clone() {
            Some(thing) => thing.id,
            None => bail!("No id provided"),
        };

        let contact: Option<WalletkaContact> = self
            .database
            .update((TABLE_NAME, id))
            .content(contact)
            .await?;

        match contact {
            Some(contact) => Ok(contact),
            None => bail!("Contact not found!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::entities::{ContactAddress, ContactAddressType};

    use super::*;
    use anyhow::Result;
    use surrealdb::engine::local::Mem;

    #[tokio::test]
    async fn create_contact() -> Result<()> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;

        let repo = ContactsRepository::new(db);

        let contact = WalletkaContact {
            id: None,
            display_name: "Test contact".to_string(),
            addresses: vec![ContactAddress {
                address_type: ContactAddressType::Nip05,
                value: "test@test.com".to_string(),
            }],
        };

        let created = repo.add(contact).await?;
        assert_ne!(None, created.id);

        let contacts = repo.get_all().await?;
        assert_eq!(1, contacts.len());

        Ok(())
    }

    #[tokio::test]
    async fn update_contact() -> Result<()> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;

        let repo = ContactsRepository::new(db);

        let contact = WalletkaContact {
            id: None,
            display_name: "Test contact".to_string(),
            addresses: vec![ContactAddress {
                address_type: ContactAddressType::Nip05,
                value: "test@test.com".to_string(),
            }],
        };

        let mut created = repo.add(contact).await?;
        assert_ne!(None, created.id);

        created.display_name = "Testing contact".to_string();

        let updated = repo.update(created.clone()).await?;
        assert_eq!(created.id, updated.id);
        assert_eq!(updated.display_name, "Testing contact".to_string());

        Ok(())
    }
}

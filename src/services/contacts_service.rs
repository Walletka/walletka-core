use anyhow::Result;
use log::{debug, info};
use nostr_sdk::{FromBech32, PublicKey, ToBech32};
use surrealdb::Connection;

use crate::io::{
    clients::NostrClient,
    entities::{ContactAddress, ContactAddressType, WalletkaContact},
    repositories::contacts_repository::ContactsRepository,
};

pub struct ContactsService<C>
where
    C: Connection,
{
    repository: ContactsRepository<C>,
    nostr_client: NostrClient,
    // Todo backup
}

impl<C> ContactsService<C>
where
    C: Connection,
{
    pub fn new(repository: ContactsRepository<C>, nostr_client: NostrClient) -> Self {
        Self {
            repository,
            nostr_client,
        }
    }
}

impl<C> ContactsService<C>
where
    C: Connection,
{
    pub async fn add(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        debug!("Creating contact {}", contact.display_name);

        let created = self.repository.add(contact).await?;

        info!(
            "Created contact {} with id {}",
            created.display_name,
            created.id.clone().unwrap().id
        );

        Ok(created)
    }

    pub async fn get_all(&self) -> Result<Vec<WalletkaContact>> {
        debug!("Getting all contacts");

        let contacts = self.repository.get_all().await?;

        Ok(contacts)
    }

    pub async fn update(&self, contact: WalletkaContact) -> Result<WalletkaContact> {
        debug!("Updating contact {}", contact.id.clone().unwrap().id);

        let updated = self.repository.update(contact).await?;

        info!("Contact {} updated", updated.id.clone().unwrap().id);

        Ok(updated)
    }

    pub async fn delete(&self, contact: WalletkaContact) -> Result<()> {
        let contact_id = contact.id.clone().unwrap().id;
        debug!("Deleting contact {}", contact_id);

        self.repository.delete(contact).await?;

        info!("Contact {} deleted", contact_id);

        Ok(())
    }

    pub async fn import_from_npub(&self, npub: String) -> Result<Vec<String>> {
        debug!("Importing contacts from npub: {}", npub);

        let contacts = self
            .nostr_client
            .get_contact_list(PublicKey::from_bech32(npub)?)
            .await?;

        let mut imported_contacts = vec![];

        for contact in contacts {
            let npub = contact.public_key.to_bech32().unwrap();

            debug!("Importing contact {npub}");

            let mut walletka_contact = WalletkaContact {
                id: None,
                display_name: contact.alias.clone().unwrap_or(npub.clone()),
                addresses: vec![ContactAddress {
                    address_type: ContactAddressType::Npub,
                    value: npub.clone(),
                }],
            };

            let metadata = self
                .nostr_client
                .get_metadata(contact.public_key.clone())
                .await?;

            walletka_contact.apply_nostr_metadata(metadata);

            self.repository.add(walletka_contact).await?;

            imported_contacts.push(npub);
        }

        info!("{} contacts imported", imported_contacts.len());

        Ok(imported_contacts)
    }
}

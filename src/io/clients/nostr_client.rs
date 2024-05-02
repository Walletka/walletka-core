use std::time::Duration;

use anyhow::Result;
use log::{debug, info};
use nostr_sdk::{
    nips::{nip04, nip06::FromMnemonic},
    Client, Contact, Event, Filter, Keys, Kind, Metadata, PublicKey, Tag, Timestamp,
    ToBech32,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(1);

pub struct NostrClient {
    keys: Keys,
    client: Client,
    pub pub_key: PublicKey,
}

impl NostrClient {
    pub async fn new(
        client_urls: Vec<String>,
        mnemonic: String,
        passphrase: Option<String>,
    ) -> Result<Self> {
        let keys = Keys::from_mnemonic(mnemonic, passphrase)?;

        let client = Client::new(&keys);

        for client_url in client_urls {
            client.add_relay(client_url).await?;
        }

        client.connect().await;

        debug!("Nostr client connected");

        Ok(Self {
            pub_key: keys.clone().public_key(),
            keys,
            client,
        })
    }

    pub async fn get_metadata(&self, pub_key: PublicKey) -> Result<Metadata> {
        debug!("Getting my metadata");

        let metadata = self.client.metadata(pub_key).await?;

        debug!("Metadata retrieved");

        Ok(metadata)
    }

    pub async fn update_metadata(&self, metadata: Metadata) -> Result<()> {
        debug!("Updating metadata\n{:#?}", metadata);

        let res = self.client.set_metadata(&metadata).await?;

        info!("Metadata updated in event:{}", res);

        Ok(())
    }

    pub async fn get_contact_list(&self, pub_key: PublicKey) -> Result<Vec<Contact>> {
        debug!("Getting {} contact list", pub_key.to_bech32()?);

        let mut contact_list: Vec<Contact> = Vec::new();
        let filters: Vec<Filter> = vec![Filter::new()
            .author(pub_key)
            .kind(Kind::ContactList)
            .limit(1)];

        let events: Vec<Event> = self
            .client
            .get_events_of(filters, Some(DEFAULT_TIMEOUT))
            .await?;

        debug!("Contact list successfully retrieved");

        for event in events.into_iter() {
            for tag in event.into_iter_tags() {
                if let Tag::PublicKey {
                    public_key,
                    relay_url,
                    alias,
                    uppercase: false,
                } = tag
                {
                    contact_list.push(Contact::new(public_key, relay_url, alias))
                }
            }
        }

        Ok(contact_list)
    }

    pub async fn get_nip04_messages(&self, since: Timestamp) -> Result<()> {
        debug!("Getting nip04 private messages");

        let filters: Vec<Filter> = vec![Filter::new()
            .kind(Kind::EncryptedDirectMessage)
            .pubkey(self.keys.public_key())
            .since(since)];

        let events = self
            .client
            .get_events_of(filters, Some(DEFAULT_TIMEOUT))
            .await?;

        debug!("Nip04 messages sucessfully retrieved");

        for event in events.into_iter() {
            let encrypted_msg = self.decrypt_nip04(event.content.clone())?;
            debug!("Received message: {}", encrypted_msg);
        }

        Ok(())
    }

    pub fn decrypt_nip04(&self, text: String) -> Result<String> {
        let decrypted_msg = nip04::decrypt(self.keys.secret_key()?, &self.keys.public_key(), text)?;
        Ok(decrypted_msg)
    }

    pub fn encrypt_nip04(&self, text: String) -> Result<String> {
        let encrypted_msg = nip04::encrypt(self.keys.secret_key()?, &self.pub_key, text)?;
        Ok(encrypted_msg)
    }
}

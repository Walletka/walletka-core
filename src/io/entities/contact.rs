use log::debug;
use nostr_sdk::Metadata;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Clone)]
pub enum ContactAddressType {
    Npub,
    Nip05,
    LightningNodePubkey,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactAddress {
    pub address_type: ContactAddressType,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletkaContact {
    pub id: Option<Thing>,
    pub display_name: String,
    pub addresses: Vec<ContactAddress>,
    // Todo
}

impl WalletkaContact {
    pub fn apply_nostr_metadata(&mut self, metadata: Metadata) {
        debug!("Setting contact details from nostr metadata");

        if let Some(name) = metadata.name {
            debug!("Setting name from nostr name: {}", name);

            self.display_name = name;
        }

        if let Some(name) = metadata.display_name {
            debug!("Setting name from nostr display name: {}", name);

            self.display_name = name;
        }

        if let Some(nip05) = metadata.nip05 {
            debug!("Setting nip05: {}", nip05);

            self.addresses.push(ContactAddress {
                address_type: ContactAddressType::Nip05,
                value: nip05,
            })
        }
        // Todo: get more info from metadata
    }
}

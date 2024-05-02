use anyhow::{bail, Result};
use cashu_sdk::{
    client::{minreq_client::HttpClient, Client},
    nuts::{Id, Proof, Token},
    url::UncheckedUrl,
    wallet::Wallet,
    Amount,
};
use itertools::Itertools;
use log::{debug, info};
use nostr_sdk::Url;
use std::{collections::HashMap, str::FromStr};
use surrealdb::{sql::Datetime, Connection};

use crate::io::{
    entities::{CashuMint, CashuProof, PendingCashuToken, PendingCashuTokenSource},
    repositories::cashu_repository::CashuRepository,
};

pub struct CashuWallet<C>
where
    C: Connection,
{
    client: HttpClient,
    repository: CashuRepository<C>,
    mints: HashMap<String, CashuMint>,
}

impl<C> CashuWallet<C>
where
    C: Connection,
{
    pub async fn new(repository: CashuRepository<C>) -> Result<Self> {
        let client = HttpClient {};

        let mut wallet = Self {
            client,
            repository,
            mints: HashMap::new(),
        };

        wallet.reload_mints().await?;

        Ok(wallet)
    }

    async fn reload_mints(&mut self) -> Result<()> {
        let mints = self.repository.get_mints().await?;

        debug!("Loaded {} mints", mints.len());

        self.mints.clear();

        for mint in mints {
            self.mints.insert(mint.keyset_id.clone(), mint);
        }

        Ok(())
    }

    async fn get_mint_wallet(&self, mint_url: UncheckedUrl) -> Result<Wallet<HttpClient>> {
        let mint_keys = self
            .client
            .get_mint_keys(Url::from_str(&mint_url.to_string())?)
            .await?;

        let wallet = Wallet::new(self.client.clone(), mint_url.clone(), mint_keys);

        Ok(wallet)
    }

    async fn add_mint(&mut self, mint: CashuMint) -> Result<CashuMint> {
        if self.mints.contains_key(&mint.keyset_id) {
            return Ok(self.mints[&mint.keyset_id].to_owned());
        }

        let mint = self.repository.add_mint(mint).await?;
        self.mints.insert(mint.keyset_id.clone(), mint.clone());

        Ok(mint)
    }

    pub fn get_mints(&self) -> Vec<CashuMint> {
        let mints: Vec<CashuMint> = self.mints.iter().map(|m| m.1.to_owned()).collect();
        mints
    }

    pub async fn claim_token(&mut self, token: String) -> Result<()> {
        debug!("Claiming cashu token...");

        let decoded_token = Token::from_str(&token)?;
        let _memo = decoded_token.memo;
        let proofs = decoded_token.token;
        let mint_url = proofs.first().unwrap().mint.clone();

        let wallet = self.get_mint_wallet(mint_url.clone()).await?;
        let new_proofs = match wallet.receive(&token).await {
            Ok(proofs) => proofs,
            Err(err) => {
                log::error!("{}", err);
                bail!("Can't claim token!")
            }
        };

        let amount = self
            .store_proofs(mint_url.clone().to_string(), new_proofs)
            .await?;

        info!("Claimed {} sats from {}", amount, mint_url);

        // Todo: store tx

        Ok(())
    }

    pub async fn get_proofs(&self) -> Result<Vec<CashuProof>> {
        self.repository.get_proofs().await
    }

    async fn store_proofs(&mut self, mint_url: String, proofs: Vec<Proof>) -> Result<u64> {
        let mut amount = 0;
        for proof in proofs {
            let id = proof.id.clone().unwrap();
            if !self.mints.contains_key(&id.to_string()) {
                self.add_mint(CashuMint {
                    keyset_id: id.to_string(),
                    mint_url: mint_url.clone(),
                    trust_level: 1,
                })
                .await?;
            }

            amount += proof.amount.to_sat();
            self.repository
                .store_proof(CashuProof::from(&proof))
                .await?;
        }

        Ok(amount)
    }

    pub async fn get_pending_tokens(&self) -> Result<Vec<PendingCashuToken>> {
        let tokens = self.repository.get_pending_tokens().await?;

        Ok(tokens)
    }

    pub async fn create_token_from_keyset(
        &mut self,
        keyset_id: String,
        amount_sat: u64,
        memo: Option<String>,
    ) -> Result<Token> {
        let proofs = self.get_proofs().await?;
        let mint = &self.mints[&keyset_id];

        let mut selected_proofs: Vec<CashuProof> = vec![];
        let mut value_to_send = 0;

        let sorted_proofs: Vec<CashuProof> = proofs
            .into_iter()
            .filter(|p| {
                p.keyset_id.is_some()
                    && p.keyset_id.unwrap() == Id::try_from_base64(&keyset_id).unwrap()
            })
            .sorted_by(|p1, p2| p1.amount_sat.partial_cmp(&p2.amount_sat).unwrap())
            .collect();

        for proof in sorted_proofs {
            debug!(
                "Adding proof {} with value {} sats",
                proof.secret.to_string(),
                proof.amount_sat
            );

            value_to_send += proof.amount_sat;
            selected_proofs.push(proof);

            if value_to_send >= amount_sat {
                break;
            }
        }

        info!("Selected amount to send: {} sats", value_to_send);

        let wallet = self
            .get_mint_wallet(UncheckedUrl::new(mint.mint_url.clone()))
            .await?;

        let result = wallet
            .send(
                Amount::from_sat(amount_sat),
                selected_proofs.iter().map(|p| p.into()).collect(),
            )
            .await?;

        for used_proof in selected_proofs {
            self.repository
                .delete_proof(used_proof.id.unwrap().id.to_string())
                .await?;
        }

        let token_to_send = Token::new(
            UncheckedUrl::new(mint.mint_url.clone()),
            result.send_proofs,
            memo,
        )?;

        // Store sent token
        self.repository
            .add_pending_token(PendingCashuToken {
                id: None,
                claimed: false,
                datetime: Datetime::default(),
                token: token_to_send.clone().convert_to_string().unwrap(),
                source: PendingCashuTokenSource::Sent,
                amount_sat,
            })
            .await?;

        let amount = self
            .store_proofs(mint.mint_url.clone(), result.change_proofs)
            .await?;

        info!("Changed amount: {} sats", amount);

        Ok(token_to_send)
    }
}

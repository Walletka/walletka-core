use log::debug;
use sha256::digest;

use anyhow::{bail, Result};
use surrealdb::{Connection, Surreal};

use crate::io::entities::{CashuMint, CashuProof, PendingCashuToken};

const CASHU_PROOFS_TABLE: &str = "cashu_proofs";
const PENDING_CASHU_TOKENS_TABLE: &str = "cashu_pending_tokens";
const CASHU_MINTS_TABLE: &str = "cashu_mints";

pub struct CashuRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> CashuRepository<C>
where
    C: Connection,
{
    pub fn new(db: Surreal<C>) -> Self {
        Self { db }
    }

    pub async fn store_proof(&self, proof: CashuProof) -> Result<CashuProof> {
        debug!("Storing proof to db...");
        let id = digest(proof.secret.to_string());
        let proof: Option<CashuProof> = self
            .db
            .create((CASHU_PROOFS_TABLE, id))
            .content(proof)
            .await?;

        match proof {
            Some(proof) => Ok(proof.to_owned()),
            None => bail!("Can't store proof!"),
        }
    }

    pub async fn get_proofs(&self) -> Result<Vec<CashuProof>> {
        let proofs: Vec<CashuProof> = self.db.select(CASHU_PROOFS_TABLE).await?;

        Ok(proofs)
    }

    pub async fn get_proof_by_id(&self, id: String) -> Result<Option<CashuProof>> {
        let proof: Option<CashuProof> = self.db.select((CASHU_PROOFS_TABLE, &id)).await?;

        Ok(proof)
    }

    pub async fn delete_proof(&self, id: String) -> Result<bool> {
        let proof: Option<CashuProof> = self.db.delete((CASHU_PROOFS_TABLE, &id)).await?;

        Ok(proof.is_some())
    }

    pub async fn add_pending_token(&self, token: PendingCashuToken) -> Result<PendingCashuToken> {
        debug!("Adding pending token");

        let id = digest(token.token.clone());

        let proof: Option<PendingCashuToken> = self
            .db
            .create((PENDING_CASHU_TOKENS_TABLE, id))
            .content(token)
            .await?;

        match proof {
            Some(proof) => Ok(proof.to_owned()),
            None => bail!("Can't store proof!"),
        }
    }

    pub async fn get_pending_tokens(&self) -> Result<Vec<PendingCashuToken>> {
        let tokens: Vec<PendingCashuToken> = self.db.select(PENDING_CASHU_TOKENS_TABLE).await?;

        Ok(tokens)
    }

    pub async fn set_pending_token_claimed(&self, id: String) -> Result<()> {
        debug!("Setting pending token claimed: {}", id);
        let mut updated = self
            .db
            .query(format!(
                "UPDATE {}:{} SET claimed = true",
                PENDING_CASHU_TOKENS_TABLE, id
            ))
            .await?;

        let token: Option<PendingCashuToken> = updated.take(0)?;

        match token {
            Some(_) => Ok(()),
            None => bail!("Can't set token claimed!"),
        }
    }

    pub async fn add_mint(&self, cashu_mint: CashuMint) -> Result<CashuMint> {
        debug!("Adding cashu mint {}", cashu_mint.keyset_id);

        let created: Option<CashuMint> = self
            .db
            .create((CASHU_MINTS_TABLE, &cashu_mint.keyset_id))
            .content(cashu_mint)
            .await?;

        match created {
            Some(mint) => Ok(mint.to_owned()),
            None => bail!("Can't create cashu mint!"),
        }
    }

    pub async fn get_mints(&self) -> Result<Vec<CashuMint>> {
        let mints: Vec<CashuMint> = self.db.select(CASHU_MINTS_TABLE).await?;

        Ok(mints)
    }

    pub async fn get_mint_by_id(&self, id: String) -> Result<Option<CashuMint>> {
        let mint: Option<CashuMint> = self.db.select((CASHU_MINTS_TABLE, id)).await?;

        Ok(mint)
    }

    pub async fn update_mint(&self, cashu_mint: CashuMint) -> Result<CashuMint> {
        let updated: Option<CashuMint> = self
            .db
            .update((CASHU_MINTS_TABLE, &cashu_mint.keyset_id))
            .content(cashu_mint)
            .await?;

        match updated {
            Some(mint) => Ok(mint),
            None => bail!("Can't update cashu mint!"),
        }
    }

    pub async fn delete_mint(&self, id: String) -> Result<()> {
        debug!("Deleting cashu mint {}", id);
        let deleted: Option<CashuMint> = self.db.delete((CASHU_MINTS_TABLE, id)).await?;

        match deleted {
            Some(_) => Ok(()),
            None => bail!("Can't delete cashu mint!"),
        }
    }
}

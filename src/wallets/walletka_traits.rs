use anyhow::Result;

pub trait NestedWallet {
    fn sync(&self) -> Result<()>;
}

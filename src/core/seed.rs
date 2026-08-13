use crate::core::error::{HdError, Result};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterSeed(pub Vec<u8>);

impl MasterSeed {
    pub fn random_32() -> Self {
        let mut b = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut b);
        Self(b.to_vec())
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self> {
        if b.len() < 16 {
            return Err(HdError::InvalidSeed("need at least 16 bytes".into()));
        }
        Ok(Self(b.to_vec()))
    }

    pub fn from_hex(s: &str) -> Result<Self> {
        let b = hex::decode(s).map_err(|_| HdError::InvalidSeed("invalid hex".into()))?;
        Self::from_bytes(&b)
    }

    pub fn read_file(p: &std::path::Path) -> Result<Self> {
        Self::from_bytes(&std::fs::read(p)?)
    }

    pub fn write_file(&self, p: &std::path::Path) -> Result<()> {
        std::fs::write(p, &self.0)?;
        Ok(())
    }

    #[cfg(feature = "mnemonic")]
    pub fn from_mnemonic(m: &str, passphrase: &str) -> Result<Self> {
        use bip39::Mnemonic;
        let m = Mnemonic::parse(m).map_err(|e| HdError::InvalidSeed(e.to_string()))?;
        Self::from_bytes(&m.to_seed(passphrase))
    }
}

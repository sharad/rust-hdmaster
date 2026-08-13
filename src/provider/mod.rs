


pub mod registry;
pub mod bip32;
pub mod slip10;

pub use registry::{derive_child, ProviderRegistry};

use crate::core::{
    error::{HdError, Result},
    node::HdNode,
    path::ChildIndex,
};
use hmac::{Hmac, Mac};
use sha2::Sha512;

use std::path::Path;

use serde::{Deserialize, Serialize};
use std::str::FromStr;




#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Algorithm {
    Secp256k1,
    Ed25519,
    P256,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationScheme {
    Bip32,
    Slip10,
}
pub type Variant = String;

pub const VARIANT_STANDARD: &str = "standard";
pub const VARIANT_BITCOIN: &str = "bitcoin";





impl FromStr for Algorithm {
    type Err = HdError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "secp256k1" => Ok(Self::Secp256k1),
            "ed25519" => Ok(Self::Ed25519),
            "p256" | "secp256r1" | "nistp256" => Ok(Self::P256),
            _ => Err(HdError::UnsupportedAlgorithm(s.into())),
        }
    }
}

impl FromStr for DerivationScheme {
    type Err = HdError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "bip32" => Ok(Self::Bip32),
            "slip10" => Ok(Self::Slip10),
            _ => Err(HdError::UnsupportedDerivationScheme(s.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderId {
    pub algorithm: Algorithm,
    pub scheme: DerivationScheme,
    pub variant: Variant,
}


pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    // fn algorithm(&self) -> Algorithm;
    // fn scheme(&self) -> DerivationScheme;
    fn supports_non_hardened(&self) -> bool;

    fn master(&self, application: &str, seed: &[u8]) -> Result<HdNode>;

    fn child(
        &self,
        parent: &HdNode,
        index: ChildIndex,
    ) -> Result<HdNode>;

    fn write_private(
        &self,
        node: &HdNode,
        path: &Path,
    ) -> Result<()>;

    fn write_public(
        &self,
        node: &HdNode,
        path: &Path,
    ) -> Result<()>;
}

pub(crate) fn hmac_sha512(
    k: &[u8],
    d: &[u8],
) -> Result<[u8; 64]> {
    type HmacSha512 = Hmac<Sha512>;
    let mut m =
        HmacSha512::new_from_slice(k)
            .map_err(|e| HdError::Crypto(e.to_string()))?;

    m.update(d);

    Ok(m.finalize().into_bytes().into())
}

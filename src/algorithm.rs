use crate::error::{HdError, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Algorithm {
    Secp256k1,
    Ed25519,
    P256,
}
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

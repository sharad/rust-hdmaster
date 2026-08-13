




use crate::{algorithm::Algorithm, error::Result, key::KeyMaterial, path::ChildIndex};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};





#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationScheme {
    Bip32,
    Slip10,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HdNode {
    pub application: String,
    #[zeroize(skip)]
    pub algorithm: Algorithm,
    #[zeroize(skip)]
    pub scheme: DerivationScheme,
    #[zeroize(skip)]
    pub depth: u32,
    #[zeroize(skip)]
    pub child_index: u32,
    pub chain_code: [u8; 32],
    pub private_key: [u8; 32],
    pub public_key: Vec<u8>,
}

impl std::fmt::Debug for HdNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HdNode")
            .field("application", &self.application)
            .field("algorithm", &self.algorithm)
            .field("scheme", &self.scheme)
            .field("depth", &self.depth)
            .field("child_index", &self.child_index)
            .field("private_key", &"<secret>")
            .field("chain_code", &"<secret>")
            .field("public_key", &hex::encode(&self.public_key))
            .finish()
    }
}

impl HdNode {

    pub fn child(&self, index: ChildIndex) -> Result<Self> {
        crate::provider::derive_child(self, index)
    }

    pub fn key_material(&self) -> KeyMaterial {
        KeyMaterial {
            algorithm: self.algorithm,
            private_key: self.private_key.to_vec(),
            public_key: self.public_key.clone(),
        }
    }
}



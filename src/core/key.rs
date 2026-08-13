use crate::ProviderId;
use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Serialize, Deserialize)]

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub provider: ProviderId,
    // pub private_key: [u8; 32],
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyMaterial")
            .field("algorithm", &self.provider.algorithm)
            .field("private_key", &"<secret>")
            .field("public_key", &hex::encode(&self.public_key))
            .finish()
    }
}

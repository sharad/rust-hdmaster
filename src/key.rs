

use crate::Algorithm;
use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub algorithm: Algorithm,
    // pub private_key: [u8; 32],
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

use serde::{Deserialize, Serialize};
use crate::Algorithm;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial { pub algorithm: Algorithm, pub private_key: Vec<u8>, pub public_key: Vec<u8> }

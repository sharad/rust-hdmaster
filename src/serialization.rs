

use crate::{error::Result, node::HdNode};
use std::path::Path;



pub fn save_node(n: &HdNode, p: &Path) -> Result<()> {
    std::fs::write(p, serde_json::to_vec_pretty(n)?)?;
    Ok(())
}

pub fn load_node(p: &Path) -> Result<HdNode> {
    Ok(serde_json::from_slice(&std::fs::read(p)?)?)
}

pub fn write_private_pem(n: &HdNode, p: &Path) -> Result<()> {
    use pkcs8::EncodePrivateKey;
    match n.algorithm {
        crate::Algorithm::Ed25519 => {
            let a: [u8; 32] = n
                .private_key
                .as_slice()
                .try_into()
                .map_err(|_| crate::HdError::InvalidPrivateKey)?;
            let k = ed25519_dalek::SigningKey::from_bytes(&a);
            let pem = k
                .to_pkcs8_pem(pkcs8::LineEnding::LF)
                .map_err(|e| crate::HdError::Crypto(e.to_string()))?;
            std::fs::write(p, pem.as_bytes())?
        }
        crate::Algorithm::P256 => {
            let k = p256::SecretKey::from_slice(&n.private_key)
                .map_err(|_| crate::HdError::InvalidPrivateKey)?;
            let pem = k
                .to_pkcs8_pem(pkcs8::LineEnding::LF)
                .map_err(|e| crate::HdError::Crypto(e.to_string()))?;
            std::fs::write(p, pem.as_bytes())?
        }
        crate::Algorithm::Secp256k1 => {
            std::fs::write(p, format!("{}\n", hex::encode(&n.private_key)))?
        }
    }
    Ok(())
}

pub fn write_public_pem(n: &HdNode, p: &Path) -> Result<()> {
    use pkcs8::{EncodePublicKey, LineEnding};
    match n.algorithm {
        crate::Algorithm::Ed25519 => {
            let a: [u8; 32] = n
                .public_key
                .as_slice()
                .try_into()
                .map_err(|_| crate::HdError::InvalidPrivateKey)?;
            let k = ed25519_dalek::VerifyingKey::from_bytes(&a)
                .map_err(|_| crate::HdError::InvalidPrivateKey)?;
            std::fs::write(
                p,
                k.to_public_key_pem(LineEnding::LF)
                    .map_err(|e| crate::HdError::Crypto(e.to_string()))?,
            )?
        }
        crate::Algorithm::P256 => {
            let k = p256::PublicKey::from_sec1_bytes(&n.public_key)
                .map_err(|_| crate::HdError::InvalidPrivateKey)?;
            std::fs::write(
                p,
                k.to_public_key_pem(LineEnding::LF)
                    .map_err(|e| crate::HdError::Crypto(e.to_string()))?,
            )?
        }
        crate::Algorithm::Secp256k1 => {
            std::fs::write(p, format!("{}\n", hex::encode(&n.public_key)))?
        }
    }
    Ok(())
}



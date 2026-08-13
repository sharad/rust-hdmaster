
// use hmac::{Hmac, Mac};
// use sha2::Sha512;
// use std::path::Path;
// type HmacSha512 = Hmac<Sha512>;


use crate::{
    provider::Algorithm,
    provider::DerivationScheme,
    provider::ProviderId,
    error::{HdError, Result},
    node::HdNode,
    path::ChildIndex,
};
use std::path::Path;

use crate::provider::{hmac_sha512, Provider, VARIANT_STANDARD};



pub(crate) struct Ed25519;

impl Ed25519 {
    fn public_key(private_key: &[u8]) -> Result<Vec<u8>> {
        let key: [u8; 32] = private_key
            .try_into()
            .map_err(|_| HdError::InvalidPrivateKey)?;

        Ok(
            ed25519_dalek::SigningKey::from_bytes(&key)
                .verifying_key()
                .to_bytes()
                .to_vec()
        )
    }
}
impl Provider for Ed25519 {
    fn id(&self) -> ProviderId {
        ProviderId {
            algorithm: Algorithm::Ed25519,
            scheme: DerivationScheme::Slip10,
            variant: VARIANT_STANDARD.into(),
        }
    }
   fn supports_non_hardened(&self) -> bool {
        false
    }
    fn master(&self, a: &str, seed: &[u8]) -> Result<HdNode> {
        let i = hmac_sha512(b"ed25519 seed", seed)?;
        Ok(HdNode {
            application: a.into(),
            provider: self.id(),
            depth: 0,
            child_index: 0,
            chain_code: i[32..].try_into().unwrap(),
            private_key: i[..32].try_into().map_err(|_| HdError::InvalidPrivateKey)?,
            public_key: Self::public_key(&i[..32])?,
        })
    }
    fn child(&self, p: &HdNode, x: ChildIndex) -> Result<HdNode> {
        if !x.is_hardened() {
            return Err(HdError::NonHardenedUnsupported);
        }
        let mut d = vec![0];
        d.extend(&p.private_key);
        d.extend(x.raw().to_be_bytes());
        let i = hmac_sha512(&p.chain_code, &d)?;
        Ok(HdNode {
            application: p.application.clone(),
            provider: self.id(),
            depth: p.depth + 1,
            child_index: x.raw(),
            chain_code: i[32..].try_into().unwrap(),
            private_key: i[..32].try_into().map_err(|_| HdError::InvalidPrivateKey)?,
            public_key: Self::public_key(&i[..32])?,
        })
    }


    fn write_private(&self, n: &HdNode, p: &Path) -> Result<()> {
        use pkcs8::EncodePrivateKey;

        let a: [u8; 32] = n
            .private_key
            .as_slice()
            .try_into()
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let k = ed25519_dalek::SigningKey::from_bytes(&a);

        let pem = k
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .map_err(|e| HdError::Crypto(e.to_string()))?;

        std::fs::write(p, pem.as_bytes())?;

        Ok(())
    }

    fn write_public(&self, n: &HdNode, p: &Path) -> Result<()> {
        use pkcs8::{EncodePublicKey, LineEnding};

        let a: [u8; 32] = n
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let k = ed25519_dalek::VerifyingKey::from_bytes(&a)
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let pem = k
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| HdError::Crypto(e.to_string()))?;

        std::fs::write(p, pem)?;

        Ok(())
    }
}




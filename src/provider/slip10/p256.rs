

use crate::provider::{Algorithm, DerivationScheme, ProviderId};

use crate::core::{
    error::{HdError, Result},
    node::HdNode,
    path::ChildIndex,
};
use std::path::Path;


use crate::provider::{hmac_sha512, Provider, VARIANT_STANDARD};



pub(crate) struct P256;
impl P256 {
    fn public_key(private_key: &[u8]) -> Result<Vec<u8>> {
        use p256::{
            elliptic_curve::sec1::ToEncodedPoint,
            SecretKey,
        };

        let sk = SecretKey::from_slice(private_key)
            .map_err(|_| HdError::InvalidPrivateKey)?;

        Ok(
            sk.public_key()
                .to_encoded_point(false)
                .as_bytes()
                .to_vec()
        )
    }
}
impl Provider for P256 {
    fn id(&self) -> ProviderId {
        ProviderId {
            algorithm: Algorithm::P256,
            scheme: DerivationScheme::Slip10,
            variant: VARIANT_STANDARD.into(),
        }
    }
    fn supports_non_hardened(&self) -> bool {
        false
    }
    fn master(&self, a: &str, seed: &[u8]) -> Result<HdNode> {
        let i = hmac_sha512(b"Nist256p1 seed", seed)?;
        let sk = p256::SecretKey::from_slice(&i[..32]).map_err(|_| HdError::InvalidPrivateKey)?;
        Ok(HdNode {
            application: a.into(),
            provider: self.id(),
            depth: 0,
            child_index: 0,
            chain_code: i[32..].try_into().unwrap(),
            private_key: sk.to_bytes().into(),
            public_key: Self::public_key(&sk.to_bytes())?,
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
        let sk = p256::SecretKey::from_slice(&i[..32]).map_err(|_| HdError::InvalidPrivateKey)?;
        Ok(HdNode {
            application: p.application.clone(),
            provider: self.id(),
            depth: p.depth + 1,
            child_index: x.raw(),
            chain_code: i[32..].try_into().unwrap(),
            private_key: sk.to_bytes().into(),
            public_key: Self::public_key(&sk.to_bytes())?,
        })
    }


    fn write_private(&self, n: &HdNode, p: &Path) -> Result<()> {
        use pkcs8::EncodePrivateKey;

        let k = p256::SecretKey::from_slice(&n.private_key)
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let pem = k
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .map_err(|e| HdError::Crypto(e.to_string()))?;

        std::fs::write(p, pem.as_bytes())?;

        Ok(())
    }

    fn write_public(&self, n: &HdNode, p: &Path) -> Result<()> {
        use pkcs8::{EncodePublicKey, LineEnding};

        let k = p256::PublicKey::from_sec1_bytes(&n.public_key)
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let pem = k
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| HdError::Crypto(e.to_string()))?;

        std::fs::write(p, pem)?;

        Ok(())
    }
}


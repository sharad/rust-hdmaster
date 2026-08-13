



use crate::{
    algorithm::Algorithm,
    error::{HdError, Result},
    node::{DerivationScheme, HdNode},
    path::ChildIndex,
};

use crate::provider::{hmac_sha512, Provider};

use std::path::Path;

// use hmac::{Hmac, Mac};
// use sha2::Sha512;
// use std::path::Path;
// type HmacSha512 = Hmac<Sha512>;


pub(crate) struct Secp;
impl Secp {
    fn public_key(private_key: &[u8]) -> Result<Vec<u8>> {
        let key: [u8; 32] = private_key
            .try_into()
            .map_err(|_| HdError::InvalidPrivateKey)?;

        let secp = secp256k1::Secp256k1::new();

        let sk = secp256k1::SecretKey::from_byte_array(key)
            .map_err(|_| HdError::InvalidPrivateKey)?;

        Ok(
            secp256k1::PublicKey::from_secret_key(&secp, &sk)
                .serialize()
                .to_vec()
        )
    }
}

impl Provider for Secp {
    fn algorithm(&self) -> Algorithm {
        Algorithm::Secp256k1
    }
    fn scheme(&self) -> DerivationScheme {
        DerivationScheme::Bip32
    }
    fn supports_non_hardened(&self) -> bool {
        true
    }

    fn master(&self, a: &str, seed: &[u8]) -> Result<HdNode> {
        let i = hmac_sha512(b"Bitcoin seed", seed)?;
        let key_bytes: [u8; 32] = i[..32]
            .try_into()
            .map_err(|_| HdError::InvalidPrivateKey)?;
        let sk =
            secp256k1::SecretKey::from_byte_array(key_bytes).map_err(|_| HdError::InvalidPrivateKey)?;
        Ok(HdNode {
            application: a.into(),
            algorithm: Algorithm::Secp256k1,
            scheme: DerivationScheme::Bip32,
            depth: 0,
            child_index: 0,
            chain_code: i[32..].try_into().unwrap(),
            private_key: sk.secret_bytes(),
            public_key: Self::public_key(&sk.secret_bytes())?,
        })
    }
    fn child(&self, p: &HdNode, x: ChildIndex) -> Result<HdNode> {
        // let secp = secp256k1::Secp256k1::new();
        let parent = secp256k1::SecretKey::from_byte_array(p.private_key)
            .map_err(|_| HdError::InvalidPrivateKey)?;
        let mut d = Vec::new();
        if x.is_hardened() {
            d.push(0);
            d.extend(&p.private_key)
        } else {
            d.extend(&p.public_key)
        }
        d.extend(x.raw().to_be_bytes());
        let i = hmac_sha512(&p.chain_code, &d)?;
        let tweak = secp256k1::Scalar::from_be_bytes(i[..32].try_into().unwrap())
            .map_err(|_| HdError::InvalidPrivateKey)?;
        let sk = parent
            .add_tweak(&tweak)
            .map_err(|_| HdError::InvalidPrivateKey)?;
        Ok(HdNode {
            application: p.application.clone(),
            algorithm: p.algorithm,
            scheme: p.scheme,
            depth: p.depth + 1,
            child_index: x.raw(),
            chain_code: i[32..].try_into().unwrap(),
            private_key: sk.secret_bytes(),
            public_key: Self::public_key(&sk.secret_bytes())?,
        })
    }



    fn write_private(&self, n: &HdNode, p: &Path) -> Result<()> {
        std::fs::write(
            p,
            format!("{}\n", hex::encode(&n.private_key)),
        )?;

        Ok(())
    }

    fn write_public(&self, n: &HdNode, p: &Path) -> Result<()> {
        std::fs::write(
            p,
            format!("{}\n", hex::encode(&n.public_key)),
        )?;

        Ok(())
    }
}


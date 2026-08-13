



use crate::{
    algorithm::Algorithm,
    error::{HdError, Result},
    node::{DerivationScheme, HdNode},
    path::ChildIndex,
};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::path::Path;
type HmacSha512 = Hmac<Sha512>;






pub trait Provider: Send + Sync {
    fn algorithm(&self) -> Algorithm;
    fn scheme(&self) -> DerivationScheme;
    fn supports_non_hardened(&self) -> bool;
    fn master(&self, application: &str, seed: &[u8]) -> Result<HdNode>;
    fn child(&self, parent: &HdNode, index: ChildIndex) -> Result<HdNode>;

    fn write_private_pem(&self, node: &HdNode, path: &Path) -> Result<()>;
    fn write_public_pem(&self, node: &HdNode, path: &Path) -> Result<()>;
}



fn hmac_sha512(k: &[u8], d: &[u8]) -> Result<[u8; 64]> {
    let mut m =
        HmacSha512::new_from_slice(k)
        .map_err(|e| HdError::Crypto(e.to_string()))?;

    m.update(d);

    let o = m.finalize().into_bytes();
    Ok(o.into())
}




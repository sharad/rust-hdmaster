

use crate::{
    error::Result,
    node::HdNode,
    provider::ProviderRegistry,
};
use std::path::Path;

pub fn save_node(n: &HdNode, p: &Path) -> Result<()> {
    std::fs::write(p, serde_json::to_vec_pretty(n)?)?;
    Ok(())
}

pub fn load_node(p: &Path) -> Result<HdNode> {
    Ok(serde_json::from_slice(&std::fs::read(p)?)?)
}

pub fn write_private_pem(n: &HdNode, p: &Path) -> Result<()> {
    ProviderRegistry::standard()
        .get(n.algorithm, n.scheme)?
        .write_private(n, p)
}

pub fn write_public_pem(n: &HdNode, p: &Path) -> Result<()> {
    ProviderRegistry::standard()
        .get(n.algorithm, n.scheme)?
        .write_public(n, p)
}






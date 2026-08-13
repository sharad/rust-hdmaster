use crate::provider::{Algorithm, DerivationScheme};

use crate::core::{
    error::{HdError, Result},
    node::HdNode,
    path::ChildIndex,
};

use super::Provider;

pub struct ProviderRegistry {
    providers: Vec<Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn standard() -> Self {
        Self {
            providers: vec![
                Box::new(crate::provider::bip32::secp::Secp),
                Box::new(crate::provider::slip10::ed25519::Ed25519),
                Box::new(crate::provider::slip10::p256::P256),
                // Box::new(Secp),
                // Box::new(Ed25519),
                // Box::new(P256),
            ],
        }
    }

    pub fn get(&self, algorithm: Algorithm, scheme: DerivationScheme) -> Result<&dyn Provider> {
        self.providers
            .iter()
            .find(|p| p.id().algorithm == algorithm && p.id().scheme == scheme)
            .map(|p| p.as_ref())
            .ok_or_else(|| HdError::UnsupportedAlgorithm(format!("{algorithm:?}/{scheme:?}")))
    }
}

pub fn derive_child(p: &HdNode, i: ChildIndex) -> Result<HdNode> {
    ProviderRegistry::standard()
        .get(p.provider.algorithm, p.provider.scheme)?
        .child(p, i)
}

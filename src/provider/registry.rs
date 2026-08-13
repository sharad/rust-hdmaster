

use super::bip32::secp::Secp;
use super::slip10::ed25519::Ed25519;
use super::slip10::p256::P256;

use crate::{
    error::{HdError, Result},
    node::{DerivationScheme, HdNode},
    algorithm::Algorithm,
    path::ChildIndex,
};

use super::Provider;

// pub struct ProviderRegistry {
//     providers: Vec<Box<dyn Provider>>,
// }

// impl ProviderRegistry {
//     pub fn standard() -> Self {
//         Self {
//             providers: vec![Box::new(Secp), Box::new(Ed25519), Box::new(P256)],
//         }
//     }
//     pub fn get(&self, algorithm: Algorithm, scheme: DerivationScheme,) -> Result<&dyn Provider> {
//         self.providers
//             .iter()
//             .find(|p| p.algorithm() == algorithm && p.scheme() == scheme)
//             .map(|p| p.as_ref())
//             .ok_or_else(|| HdError::UnsupportedAlgorithm(format!("{algorithm:?}")))
//     }
// }

// pub fn derive_child(p: &HdNode, i: ChildIndex) -> Result<HdNode> {
//     ProviderRegistry::standard().get(p.algorithm, p.scheme)?.child(p, i)
// }






pub struct ProviderRegistry {
    providers: Vec<Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn standard() -> Self {
        Self {
            providers: vec![
                Box::new(Secp),
                Box::new(Ed25519),
                Box::new(P256),
            ],
        }
    }

    pub fn get(
        &self,
        algorithm: Algorithm,
        scheme: DerivationScheme,
    ) -> Result<&dyn Provider> {
        self.providers
            .iter()
            .find(|p| {
                p.algorithm() == algorithm &&
                    p.scheme() == scheme
            })
            .map(|p| p.as_ref())
            .ok_or_else(|| {
                HdError::UnsupportedAlgorithm(
                    format!("{algorithm:?}/{scheme:?}")
                )
            })
    }
}

pub fn derive_child(
    p: &HdNode,
    i: ChildIndex,
) -> Result<HdNode> {
    ProviderRegistry::standard()
        .get(p.algorithm, p.scheme)?
        .child(p, i)
}

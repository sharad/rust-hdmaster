pub mod core;
pub mod fuse;
pub mod provider;
pub mod serialize;

pub use core::{ChildIndex, DerivationPath, HdError, HdNode, MasterSeed, NodeDeriver, Result};

pub use provider::{Provider, ProviderId, ProviderRegistry};

// pub type Result<T> = std::result::Result<T, HdError>;

// #[derive(Debug, thiserror::Error)]
// pub enum HdError {
//     #[error("I/O error: {0}")]
//     Io(
//         #[from]
//         std::io::Error),
//     #[error("unsupported: {0}")]
//     Unsupported(String)
// }

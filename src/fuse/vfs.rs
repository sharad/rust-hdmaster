use async_trait::async_trait;
use std::path::Path;

use crate::core::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualFile {
    Key,
    Pub,
}

impl VirtualFile {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "key" => Some(Self::Key),
            "pub" => Some(Self::Pub),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Pub => "pub",
        }
    }
}

/// Generates the contents of virtual files.
///
/// This is intentionally independent of FUSE.
///
/// Later this can be implemented by something that uses:
///
///     ProviderRegistry
///     HdNode
///     ProviderId
///     mnemonic/seed
///     etc.
#[async_trait]
pub trait VirtualFileProvider: Send + Sync {
    async fn generate(
        &self,
        path: &Path,
        file: VirtualFile,
    ) -> Result<Vec<u8>>;
}

/// Temporary implementation.
///
/// Later replace this with the actual HD key generator.
pub struct PlaceholderVirtualFileProvider;

#[async_trait]
impl VirtualFileProvider for PlaceholderVirtualFileProvider {
    async fn generate(
        &self,
        _path: &Path,
        _file: VirtualFile,
    ) -> Result<Vec<u8>> {
        Ok(b"... GENERATE TEXT ...\n".to_vec())
    }
}



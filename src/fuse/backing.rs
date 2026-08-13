







use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{HdError, Result};

#[async_trait]
pub trait BackingStore: Send + Sync {
    async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata>;

    async fn read(&self, path: &Path) -> Result<Vec<u8>>;

    async fn read_dir(&self, path: &Path)
                      -> Result<Vec<std::fs::DirEntry>>;

    fn real_path(&self, path: &Path) -> PathBuf;
}


pub struct LocalBackingStore {
    root: PathBuf,
}

impl LocalBackingStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    fn resolve(&self, path: &Path) -> PathBuf {
        let path = path.strip_prefix("/").unwrap_or(path);
        self.root.join(path)
    }
}

#[async_trait]
impl BackingStore for LocalBackingStore {
    async fn metadata(&self, path: &Path) -> Result<std::fs::Metadata> {
        Ok(fs::metadata(self.resolve(path)).await?)
    }

    async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(fs::read(self.resolve(path)).await?)
    }

    async fn read_dir(
        &self,
        path: &Path,
    ) -> Result<Vec<std::fs::DirEntry>> {
        let mut result = Vec::new();

        let mut entries =
            fs::read_dir(self.resolve(path)).await?;

        while let Some(entry) = entries.next_entry().await? {
            result.push(entry);
        }

        Ok(result)
    }

    fn real_path(&self, path: &Path) -> PathBuf {
        self.resolve(path)
    }
}




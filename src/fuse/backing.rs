

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::core::{HdError, Result};

#[async_trait]
pub trait BackingStore: Send + Sync {
    async fn metadata(&self, path: &Path)
        -> Result<std::fs::Metadata>;

    async fn read(&self, path: &Path)
        -> Result<Vec<u8>>;

    async fn read_dir(&self, path: &Path)
        -> Result<Vec<std::fs::DirEntry>>;

    async fn write(
        &self,
        path: &Path,
        data: &[u8],
    ) -> Result<()>;

    async fn create_dir(
        &self,
        path: &Path,
    ) -> Result<()>;

    async fn remove_file(
        &self,
        path: &Path,
    ) -> Result<()>;

    async fn remove_dir(
        &self,
        path: &Path,
    ) -> Result<()>;

    async fn rename(
        &self,
        old: &Path,
        new: &Path,
    ) -> Result<()>;

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
    async fn metadata(
        &self,
        path: &Path,
    ) -> Result<std::fs::Metadata> {
        Ok(fs::metadata(self.resolve(path)).await?)
    }

    async fn read(
        &self,
        path: &Path,
    ) -> Result<Vec<u8>> {
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

    async fn write(
        &self,
        path: &Path,
        data: &[u8],
    ) -> Result<()> {
        Ok(fs::write(self.resolve(path), data).await?)
    }

    async fn create_dir(
        &self,
        path: &Path,
    ) -> Result<()> {
        Ok(fs::create_dir(self.resolve(path)).await?)
    }

    async fn remove_file(
        &self,
        path: &Path,
    ) -> Result<()> {
        Ok(fs::remove_file(self.resolve(path)).await?)
    }

    async fn remove_dir(
        &self,
        path: &Path,
    ) -> Result<()> {
        Ok(fs::remove_dir(self.resolve(path)).await?)
    }

    async fn rename(
        &self,
        old: &Path,
        new: &Path,
    ) -> Result<()> {
        Ok(fs::rename(
            self.resolve(old),
            self.resolve(new),
        ).await?)
    }

    fn real_path(&self, path: &Path) -> PathBuf {
        self.resolve(path)
    }
}



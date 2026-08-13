use crate::{HdError, Result};
use async_trait::async_trait;
use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct DirEntryInfo {
    pub name: String,
    pub metadata: Metadata,
}

#[async_trait]
pub trait BackingStore: Send + Sync {
    async fn metadata(&self, path: &Path) -> Result<Metadata>;
    async fn read(&self, path: &Path) -> Result<Vec<u8>>;
    async fn read_range(&self, path: &Path, offset: u64, size: usize) -> Result<Vec<u8>>;
    async fn write(&self, path: &Path, offset: u64, data: &[u8]) -> Result<usize>;
    async fn create(&self, path: &Path, mode: u32) -> Result<()>;
    async fn mkdir(&self, path: &Path, mode: u32) -> Result<()>;
    async fn remove_file(&self, path: &Path) -> Result<()>;
    async fn remove_dir(&self, path: &Path) -> Result<()>;
    async fn rename(&self, old: &Path, new: &Path) -> Result<()>;
    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntryInfo>>;
    fn real_path(&self, path: &Path) -> PathBuf;
}

pub struct LocalBackingStore {
    root: PathBuf,
}
impl LocalBackingStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    fn resolve(&self, path: &Path) -> PathBuf {
        let rel = path.strip_prefix("/").unwrap_or(path);
        self.root.join(rel)
    }
    fn reject_reserved(path: &Path) -> Result<()> {
        match path.file_name().and_then(|x| x.to_str()) {
            Some("key") | Some("pub") => {
                Err(HdError::Unsupported("reserved virtual file name".into()))
            }
            _ => Ok(()),
        }
    }
}
#[async_trait]
impl BackingStore for LocalBackingStore {
    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        Ok(fs::metadata(self.resolve(path)).await?)
    }
    async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Self::reject_reserved(path)?;
        Ok(fs::read(self.resolve(path)).await?)
    }
    async fn read_range(&self, path: &Path, offset: u64, size: usize) -> Result<Vec<u8>> {
        Self::reject_reserved(path)?;
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let mut f = fs::File::open(self.resolve(path)).await?;
        f.seek(std::io::SeekFrom::Start(offset)).await?;
        let mut b = vec![0; size];
        let n = f.read(&mut b).await?;
        b.truncate(n);
        Ok(b)
    }
    async fn write(&self, path: &Path, offset: u64, data: &[u8]) -> Result<usize> {
        Self::reject_reserved(path)?;
        use tokio::io::{AsyncSeekExt, AsyncWriteExt};
        let mut f = fs::OpenOptions::new()
            .write(true)
            .open(self.resolve(path))
            .await?;
        f.seek(std::io::SeekFrom::Start(offset)).await?;
        f.write(data).await.map_err(Into::into)
    }
    async fn create(&self, path: &Path, _mode: u32) -> Result<()> {
        Self::reject_reserved(path)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.resolve(path))
            .await?;
        Ok(())
    }
    async fn mkdir(&self, path: &Path, _mode: u32) -> Result<()> {
        Self::reject_reserved(path)?;
        fs::create_dir(self.resolve(path)).await?;
        Ok(())
    }
    async fn remove_file(&self, path: &Path) -> Result<()> {
        Self::reject_reserved(path)?;
        fs::remove_file(self.resolve(path)).await?;
        Ok(())
    }
    async fn remove_dir(&self, path: &Path) -> Result<()> {
        Self::reject_reserved(path)?;
        fs::remove_dir(self.resolve(path)).await?;
        Ok(())
    }
    async fn rename(&self, old: &Path, new: &Path) -> Result<()> {
        Self::reject_reserved(old)?;
        Self::reject_reserved(new)?;
        fs::rename(self.resolve(old), self.resolve(new)).await?;
        Ok(())
    }
    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntryInfo>> {
        let mut rd = fs::read_dir(self.resolve(path)).await?;
        let mut out = Vec::new();
        while let Some(e) = rd.next_entry().await? {
            let name = e.file_name().to_string_lossy().into_owned();
            let md = e.metadata().await?;
            out.push(DirEntryInfo { name, metadata: md });
        }
        Ok(out)
    }
    fn real_path(&self, path: &Path) -> PathBuf {
        self.resolve(path)
    }
}

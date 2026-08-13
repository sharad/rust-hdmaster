



use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use fuser::{
    FileAttr,
    FileType,
    Filesystem,
    ReplyAttr,
    ReplyData,
    ReplyDirectory,
    ReplyEntry,
    Request,
};

use crate::error::Result;

use super::{
    backing::BackingStore,
    inode::{virtual_inode, ROOT_INO},
    virtual_file::VirtualFile,
};

const TTL: Duration = Duration::from_secs(1);

pub struct HdFuse<B> {
    backing: Arc<B>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl<B> HdFuse<B>
where
    B: BackingStore + 'static,
{
    pub fn new(
        backing: B,
        runtime: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            backing: Arc::new(backing),
            runtime: Arc::new(runtime),
        }
    }

    fn virtual_file_data(
        &self,
        path: &Path,
        file: VirtualFile,
    ) -> Result<Vec<u8>> {
        // Temporary implementation.
        //
        // This is where we will connect your existing
        // ProviderRegistry / HdNode / provider system.
        //
        // For now we intentionally do not generate
        // anything.

        let _ = (path, file);

        Err(crate::error::HdError::Unsupported(
            "virtual key generation not connected yet".into(),
        ))
    }

    fn is_virtual_name(name: &OsStr) -> Option<VirtualFile> {
        VirtualFile::from_name(name)
    }
}


impl<B> Filesystem for HdFuse<B>
where
    B: BackingStore + 'static,
{
    fn lookup(
        &self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: ReplyEntry,
    ) {
        if let Some(_virtual_file) =
            Self::is_virtual_name(name)
        {
            let ino = virtual_inode(parent, name.to_string_lossy().as_ref());

            let attr = FileAttr {
                ino,
                size: 0,
                blocks: 0,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: 0o400,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
                blksize: 4096,
            };

            reply.entry(&TTL, &attr, 0);
            return;
        }

        // Normal backing-file lookup will be added next.
        reply.error(libc::ENOENT);
    }



    fn read(
        &self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        // Resolve inode → virtual file.

        // Generate key/pub.

        // Return requested portion.
    }
}



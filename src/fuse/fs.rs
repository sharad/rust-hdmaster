



use std::{
    ffi::{OsStr, OsString},
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

use super::{
    backing::BackingStore,
    inode::{
        InodeKind,
        SharedInodeTable,
        ROOT_INO,
    },
    vfs::{
        VirtualFile,
        VirtualFileProvider,
    },
};

const TTL: Duration =
    Duration::from_secs(1);

pub struct HdFuse<B, V> {
    backing: Arc<B>,
    virtual_provider: Arc<V>,
    inodes: Arc<SharedInodeTable>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl<B, V> HdFuse<B, V>
where
    B: BackingStore + 'static,
    V: VirtualFileProvider + 'static,
{
    pub fn new(
        backing: B,
        virtual_provider: V,
        runtime: tokio::runtime::Runtime,
    ) -> Self {
        Self {
            backing: Arc::new(backing),
            virtual_provider: Arc::new(virtual_provider),
            inodes: Arc::new(SharedInodeTable::new()),
            runtime: Arc::new(runtime),
        }
    }

    fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(future)
    }

    fn is_virtual_name(
        name: &OsStr,
    ) -> Option<VirtualFile> {
        VirtualFile::from_name(name.to_str()?)
    }

    fn virtual_attr(
        &self,
        ino: u64,
    ) -> FileAttr {
        let now = SystemTime::now();

        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,

            kind: FileType::RegularFile,

            // Generated private key should be
            // readable only by owner.
            perm: 0o400,

            nlink: 1,
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }

    fn normal_attr(
        &self,
        ino: u64,
        metadata: &std::fs::Metadata,
    ) -> FileAttr {
        let kind =
            if metadata.is_dir() {
                FileType::Directory
            } else if metadata.is_file() {
                FileType::RegularFile
            } else {
                FileType::Symlink
            };

        FileAttr {
            ino,
            size: metadata.len(),
            blocks: metadata.blocks(),
            atime: metadata.accessed()
                .unwrap_or(SystemTime::UNIX_EPOCH),
            mtime: metadata.modified()
                .unwrap_or(SystemTime::UNIX_EPOCH),
            ctime: metadata.modified()
                .unwrap_or(SystemTime::UNIX_EPOCH),
            crtime: SystemTime::UNIX_EPOCH,
            kind,
            perm: 0o755,
            nlink: 1,
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }

    fn lookup_virtual(
        &self,
        parent: u64,
        name: &OsStr,
    ) -> Option<(u64, VirtualFile)> {
        let file = Self::is_virtual_name(name)?;

        let parent_entry =
            self.inodes.get(parent)?;

        let path =
            parent_entry.path.join(name);

        let ino = self.inodes.insert(
            parent,
            name.to_os_string(),
            path,
            InodeKind::Virtual(file),
        );

        Some((ino, file))
    }
}

impl<B, V> Filesystem for HdFuse<B, V>
where
    B: BackingStore + 'static,
    V: VirtualFileProvider + 'static,
{
    fn lookup(
        &self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: ReplyEntry,
    ) {
        /*
         * IMPORTANT:
         *
         * This is what makes:
         *
         *     cat /mount/.../key
         *
         * work even though "key" is not present
         * in the backing directory.
         */

        if let Some((ino, _file)) =
            self.lookup_virtual(parent, name)
        {
            let attr = self.virtual_attr(ino);

            reply.entry(
                &TTL,
                &attr,
                0,
            );

            return;
        }

        let parent_entry =
            match self.inodes.get(parent) {
                Some(x) => x,
                None => {
                    reply.error(libc::ENOENT);
                    return;
                }
            };

        let child_path =
            parent_entry.path.join(name);

        let metadata =
            match self.block_on(
                self.backing.metadata(&child_path),
            ) {
                Ok(x) => x,
                Err(_) => {
                    reply.error(libc::ENOENT);
                    return;
                }
            };

        let ino = self.inodes.insert(
            parent,
            name.to_os_string(),
            child_path,
            InodeKind::Normal,
        );

        let attr =
            self.normal_attr(ino, &metadata);

        reply.entry(
            &TTL,
            &attr,
            0,
        );
    }

    fn getattr(
        &self,
        _req: &Request<'_>,
        ino: u64,
        _fh: Option<u64>,
        reply: ReplyAttr,
    ) {
        let entry =
            match self.inodes.get(ino) {
                Some(x) => x,
                None => {
                    reply.error(libc::ENOENT);
                    return;
                }
            };

        match entry.kind {
            InodeKind::Virtual(_) => {
                let attr =
                    self.virtual_attr(ino);

                reply.attr(
                    &TTL,
                    &attr,
                );
            }

            InodeKind::Root |
            InodeKind::Normal => {
                let metadata =
                    match self.block_on(
                        self.backing.metadata(
                            &entry.path,
                        ),
                    ) {
                        Ok(x) => x,
                        Err(_) => {
                            reply.error(
                                libc::ENOENT,
                            );
                            return;
                        }
                    };

                let attr =
                    self.normal_attr(
                        ino,
                        &metadata,
                    );

                reply.attr(
                    &TTL,
                    &attr,
                );
            }
        }
    }

    fn readdir(
        &self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let entry =
            match self.inodes.get(ino) {
                Some(x) => x,
                None => {
                    reply.error(libc::ENOENT);
                    return;
                }
            };

        if matches!(
            entry.kind,
            InodeKind::Virtual(_)
        ) {
            reply.error(libc::ENOTDIR);
            return;
        }

        /*
         * The important part:
         *
         * backing directory is read normally,
         * but "key" and "pub" are filtered.
         *
         * Therefore:
         *
         *     ls
         *
         * DOES NOT show them.
         */

        let entries =
            match self.block_on(
                self.backing.read_dir(
                    &entry.path,
                ),
            ) {
                Ok(x) => x,
                Err(_) => {
                    reply.error(
                        libc::ENOENT,
                    );
                    return;
                }
            };

        let mut index = 0i64;

        if offset <= index {
            if reply.add(
                ino,
                index + 1,
                FileType::Directory,
                ".",
            ) {
                reply.ok();
                return;
            }
        }

        index += 1;

        if offset <= index {
            let parent =
                entry.parent;

            if reply.add(
                parent,
                index + 1,
                FileType::Directory,
                "..",
            ) {
                reply.ok();
                return;
            }
        }

        index += 1;

        for dir_entry in entries {
            let name =
                dir_entry.file_name();

            /*
             * Never expose virtual names
             * through readdir().
             */
            if Self::is_virtual_name(
                &name,
            ).is_some() {
                continue;
            }

            if index < offset {
                index += 1;
                continue;
            }

            let child_path =
                entry.path.join(&name);

            let metadata =
                match dir_entry.metadata() {
                    Ok(x) => x,
                    Err(_) => continue,
                };

            let kind =
                if metadata.is_dir() {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                };

            let child_ino =
                self.inodes.insert(
                    ino,
                    name.clone(),
                    child_path,
                    InodeKind::Normal,
                );

            if reply.add(
                child_ino,
                index + 1,
                kind,
                &name,
            ) {
                break;
            }

            index += 1;
        }

        reply.ok();
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
        let entry =
            match self.inodes.get(ino) {
                Some(x) => x,
                None => {
                    reply.error(libc::ENOENT);
                    return;
                }
            };

        let data =
            match entry.kind {
                InodeKind::Virtual(file) => {
                    match self.block_on(
                        self.virtual_provider
                            .generate(
                                &entry.path,
                                file,
                            ),
                    ) {
                        Ok(x) => x,
                        Err(_) => {
                            reply.error(
                                libc::EIO,
                            );
                            return;
                        }
                    }
                }

                InodeKind::Normal => {
                    match self.block_on(
                        self.backing.read(
                            &entry.path,
                        ),
                    ) {
                        Ok(x) => x,
                        Err(_) => {
                            reply.error(
                                libc::EIO,
                            );
                            return;
                        }
                    }
                }

                InodeKind::Root => {
                    reply.error(libc::EISDIR);
                    return;
                }
            };

        let start =
            offset.max(0) as usize;

        if start >= data.len() {
            reply.data(&[]);
            return;
        }

        let end =
            std::cmp::min(
                start + size as usize,
                data.len(),
            );

        reply.data(
            &data[start..end],
        );
    }
}




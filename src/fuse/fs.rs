use super::{
    backing::BackingStore,
    inode::{inode_for_path, ROOT_INO},
    vfs::VirtualFile,
};
use crate::{HdError, Result};

// use fuser::{
//     , FileAttr, FileHandle, FileType, Filesystem, INodeNo, OpenFlags, ReplyAttr, ReplyCreate,
//     ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, ReplyWrite, Request,
// };


use fuser::{
    Errno,
    FileAttr,
    FileType,
    Filesystem,
    Generation,
    ReplyAttr,
    ReplyCreate,
    ReplyData,
    ReplyDirectory,
    ReplyEmpty,
    ReplyEntry,
    ReplyOpen,
    ReplyWrite,
    Request,

    FileHandle,
    INodeNo,
    OpenFlags,
    FopenFlags,
};


use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

const TTL: Duration = Duration::from_secs(1);
const GENERATION: Generation = Generation(0);

pub trait VirtualGenerator: Send + Sync {
    fn generate(&self, path: &Path, file: VirtualFile) -> Result<Vec<u8>>;
}

pub struct FillerGenerator;
impl VirtualGenerator for FillerGenerator {
    fn generate(&self, path: &Path, file: VirtualFile) -> Result<Vec<u8>> {
        let n = match file {
            VirtualFile::Key => "key",
            VirtualFile::Pub => "pub",
        };
        Ok(format!(
            "... GENERATE TEXT ...\\nvirtual={n}\\npath={}\\n",
            path.display()
        )
        .into_bytes())
    }
}

struct State {
    paths: HashMap<INodeNo, PathBuf>,
}
impl State {
    fn new() -> Self {
        let mut paths = HashMap::new();
        paths.insert(ROOT_INO, PathBuf::from("/"));
        Self { paths }
    }
}

pub struct HdFuse<B, G = FillerGenerator> {
    backing: Arc<B>,
    generator: Arc<G>,
    runtime: Arc<tokio::runtime::Runtime>,
    state: Arc<Mutex<State>>,
}
impl<B, G> HdFuse<B, G>
where
    B: BackingStore + 'static,
    G: VirtualGenerator + 'static,
{
    pub fn new(backing: B, generator: G, runtime: tokio::runtime::Runtime) -> Self {
        Self {
            backing: Arc::new(backing),
            generator: Arc::new(generator),
            runtime: Arc::new(runtime),
            state: Arc::new(Mutex::new(State::new())),
        }
    }
    fn block_on<F, T>(&self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        self.runtime.block_on(f)
    }
    fn path_of(&self, ino: INodeNo) -> Option<PathBuf> {
        self.state.lock().ok()?.paths.get(&ino).cloned()
    }
    fn remember(&self, path: PathBuf) -> INodeNo {
        let ino = inode_for_path(&path);
        if let Ok(mut s) = self.state.lock() {
            s.paths.insert(ino, path);
        }
        ino
    }
    fn child_path(parent: &Path, name: &OsStr) -> PathBuf {
        let mut p = if parent == Path::new("/") {
            PathBuf::from("/")
        } else {
            parent.to_path_buf()
        };
        p.push(name);
        if !p.is_absolute() {
            let mut q = PathBuf::from("/");
            q.push(p);
            p = q;
        }
        p
    }
    fn attr(ino: INodeNo, md: &std::fs::Metadata) -> FileAttr {
        FileAttr {
            ino,
            size: md.len(),
            blocks: (md.len() + 511) / 512,
            atime: md.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
            mtime: md.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            ctime: md.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            crtime: SystemTime::UNIX_EPOCH,
            kind: if md.is_dir() {
                FileType::Directory
            } else {
                FileType::RegularFile
            },
            perm: if md.is_dir() { 0o755 } else { 0o644 },
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }
    fn vattr(&self, ino: INodeNo) -> FileAttr {
        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o400,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }
    fn errno(e: &HdError) -> Errno {
        match e {
            HdError::Io(x) => Errno::from_i32(match x.kind() {
                std::io::ErrorKind::NotFound => libc::ENOENT,
                std::io::ErrorKind::PermissionDenied => libc::EACCES,
                std::io::ErrorKind::AlreadyExists => libc::EEXIST,
                std::io::ErrorKind::NotADirectory => libc::ENOTDIR,
                _ => libc::EIO,
            }),
            _ => Errno::from_i32(libc::EIO),
        }
    }
    fn reserved(path: &Path) -> Option<VirtualFile> {
        VirtualFile::from_name(path.file_name()?)
    }
}

impl<B, G> Filesystem for HdFuse<B, G>
where
    B: BackingStore + 'static,
    G: VirtualGenerator + 'static,
{
    fn lookup(&self, _: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {

        let Some(pp) = self.path_of(parent) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        let path = Self::child_path(&pp, name);
        if let Some(v) = Self::reserved(&path) {
            let ino = self.remember(path);
            reply.entry(&TTL, &self.vattr(ino), GENERATION);
            let _ = v;
            return;
        }
        match self.block_on(self.backing.metadata(&path)) {
            Ok(md) => {
                let ino = self.remember(path);
                reply.entry(&TTL, &Self::attr(ino, &md), GENERATION)
            }
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn getattr(&self, _: &Request, ino: INodeNo, _: Option<FileHandle>, reply: ReplyAttr) {
        if ino == ROOT_INO {
            if let Ok(md) = self.block_on(self.backing.metadata(Path::new("/"))) {
                reply.attr(&TTL, &Self::attr(ino, &md));
                return;
            }
        }
        if let Some(p) = self.path_of(ino) {
            if Self::reserved(&p).is_some() {
                reply.attr(&TTL, &self.vattr(ino));
                return;
            }
            if let Ok(md) = self.block_on(self.backing.metadata(&p)) {
                reply.attr(&TTL, &Self::attr(ino, &md));
                return;
            }
        }
        reply.error(Errno::from_i32(libc::ENOENT));
    }
    fn readdir(
        &self,
        _: &Request,
        ino: INodeNo,
        _: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let Some(path) = self.path_of(ino) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        let entries = match self.block_on(self.backing.read_dir(&path)) {
            Ok(x) => x,
            Err(e) => {
                reply.error(Self::errno(&e));
                return;
            }
        };
        if offset == 0 {
            if reply.add(ino, 1, FileType::Directory, ".") {
                reply.ok();
                return;
            }
            let parent = if ino == ROOT_INO {
                ROOT_INO
            } else {
                self.remember(
                    path.parent()
                        .unwrap_or(Path::new("/"))
                        .to_path_buf()
                )
            };
            if reply.add(parent, 2, FileType::Directory, "..") {
                reply.ok();
                return;
            }
        }
        let mut idx = 3u64;
        for e in entries {
            if VirtualFile::from_name(OsStr::new(&e.name)).is_some() {
                continue;
            }
            let p = Self::child_path(&path, OsStr::new(&e.name));
            let ci = self.remember(p);
            let kind = if e.metadata.is_dir() {
                FileType::Directory
            } else {
                FileType::RegularFile
            };
            if reply.add(ci, idx, kind, e.name) {
                break;
            }
            idx += 1;
        }
        reply.ok();
    }
    fn open(&self, _: &Request,
            ino: INodeNo,
            _flags: OpenFlags,
            reply: ReplyOpen,
    ) {
        if let Some(p) = self.path_of(ino) {
            if Self::reserved(&p).is_some() {
                // reply.opened(0, flags as u32);
                reply.opened(
                    FileHandle(0),
                    FopenFlags::empty(),
                );
                return;
            }
            if self.block_on(self.backing.metadata(&p)).is_ok() {
                // reply.opened(0, flags as u32);
                reply.opened(
                    FileHandle(0),
                    FopenFlags::empty(),
                );
                return;
            }
        }
        reply.error(Errno::from_i32(libc::ENOENT));
    }
    fn read(
        &self,
        _: &Request,
        ino: INodeNo,
        _: FileHandle,
        offset: u64,
        size: u32,
        _: OpenFlags,
        _: Option<fuser::LockOwner>,
        reply: ReplyData,
    ) {
        let Some(path) = self.path_of(ino) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        let data = if let Some(v) = Self::reserved(&path) {
            match self.generator.generate(&path, v) {
                Ok(x) => x,
                Err(e) => {
                    reply.error(Self::errno(&e));
                    return;
                }
            }
        } else {
            match self.block_on(self.backing.read_range(&path, offset, size as usize)) {
                Ok(x) => {
                    reply.data(&x);
                    return;
                }
                Err(e) => {
                    reply.error(Self::errno(&e));
                    return;
                }
            }
        };
        let start = offset as usize;
        if start >= data.len() {
            reply.data(&[]);
            return;
        }
        let end = (start + size as usize).min(data.len());
        reply.data(&data[start..end]);
    }
    fn write(
        // &self,
        // _: &Request,
        // ino: INodeNo,
        // _: FileHandle,
        // offset: u64,
        // data: &[u8],
        // _: u32,
        // _: Option<fuser::LockOwner>,
        // reply: ReplyWrite,

        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        data: &[u8],
        _write_flags: fuser::WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        reply: ReplyWrite,

    ) {
        let Some(p) = self.path_of(ino) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        if Self::reserved(&p).is_some() {
            reply.error(Errno::from_i32(libc::EACCES));
            return;
        }
        match self.block_on(self.backing.write(&p, offset, data)) {
            Ok(n) => reply.written(n as u32),
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn create(
        &self,
        _: &Request,
        parent: INodeNo,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let Some(pp) = self.path_of(parent) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        if VirtualFile::from_name(name).is_some() {
            reply.error(Errno::from_i32(libc::EEXIST));
            return;
        }
        let p = Self::child_path(&pp, name);
        match self.block_on(self.backing.create(&p, mode)) {
            Ok(()) => {
                let ino = self.remember(p);
                if let Ok(md) =
                    self.block_on(self.backing.metadata(self.path_of(ino).as_deref().unwrap()))
                {
                    // reply.created(&TTL, &Self::attr(ino, &md), 0, 0, flags as u32)
                    reply.created(
                        &TTL,
                        &Self::attr(ino, &md),
                        GENERATION,
                        FileHandle(0),
                        FopenFlags::empty(),
                    )
                } else {
                    reply.error(Errno::from_i32(libc::EIO))
                }
            }
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn mkdir(
        &self,
        _: &Request,
        parent: INodeNo,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let Some(pp) = self.path_of(parent) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        if VirtualFile::from_name(name).is_some() {
            reply.error(Errno::from_i32(libc::EEXIST));
            return;
        }
        let p = Self::child_path(&pp, name);
        match self.block_on(self.backing.mkdir(&p, mode)) {
            Ok(()) => {
                let ino = self.remember(p);
                if let Ok(md) =
                    self.block_on(self.backing.metadata(self.path_of(ino).as_deref().unwrap()))
                {
                    reply.entry(&TTL, &Self::attr(ino, &md), Generation(0))
                } else {
                    reply.error(Errno::from_i32(libc::EIO))
                }
            }
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn unlink(&self, _: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        let Some(pp) = self.path_of(parent) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        if VirtualFile::from_name(name).is_some() {
            reply.error(Errno::from_i32(libc::EACCES));
            return;
        }
        match self.block_on(self.backing.remove_file(&Self::child_path(&pp, name))) {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn rmdir(&self, _: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        let Some(pp) = self.path_of(parent) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        match self.block_on(self.backing.remove_dir(&Self::child_path(&pp, name))) {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
    fn rename(
        &self,
        _: &Request,
        parent: INodeNo,
        name: &OsStr,
        newparent: INodeNo,
        newname: &OsStr,
        _flags: fuser::RenameFlags,
        reply: ReplyEmpty,
    ) {
        let (Some(a), Some(b)) = (self.path_of(parent), self.path_of(newparent)) else {
            reply.error(Errno::from_i32(libc::ENOENT));
            return;
        };
        if VirtualFile::from_name(name).is_some() || VirtualFile::from_name(newname).is_some() {
            reply.error(Errno::from_i32(libc::EACCES));
            return;
        }
        match self.block_on(
            self.backing
                .rename(&Self::child_path(&a, name), &Self::child_path(&b, newname)),
        ) {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(Self::errno(&e)),
        }
    }
}

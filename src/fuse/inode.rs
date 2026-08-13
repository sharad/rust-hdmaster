


use std::{
    collections::HashMap,
    ffi::OsString,
    path::PathBuf,
    sync::RwLock,
};

use super::vfs::VirtualFile;

pub const ROOT_INO: u64 = 1;

#[derive(Debug, Clone)]
pub enum InodeKind {
    Root,
    Normal,
    Virtual(VirtualFile),
}

#[derive(Debug, Clone)]
pub struct InodeEntry {
    pub parent: u64,
    pub name: OsString,
    pub path: PathBuf,
    pub kind: InodeKind,
}

pub struct InodeTable {
    next_inode: u64,
    entries: HashMap<u64, InodeEntry>,
    lookup: HashMap<(u64, OsString), u64>,
}

impl InodeTable {
    pub fn new() -> Self {
        let mut entries = HashMap::new();

        entries.insert(
            ROOT_INO,
            InodeEntry {
                parent: ROOT_INO,
                name: OsString::from("/"),
                path: PathBuf::from("/"),
                kind: InodeKind::Root,
            },
        );

        Self {
            next_inode: ROOT_INO + 1,
            entries,
            lookup: HashMap::new(),
        }
    }

    pub fn get(&self, ino: u64) -> Option<&InodeEntry> {
        self.entries.get(&ino)
    }

    pub fn find(
        &self,
        parent: u64,
        name: &OsString,
    ) -> Option<u64> {
        self.lookup
            .get(&(parent, name.clone()))
            .copied()
    }

    pub fn insert(
        &mut self,
        parent: u64,
        name: OsString,
        path: PathBuf,
        kind: InodeKind,
    ) -> u64 {
        if let Some(ino) =
            self.lookup.get(&(parent, name.clone()))
        {
            return *ino;
        }

        let ino = self.next_inode;
        self.next_inode += 1;

        self.entries.insert(
            ino,
            InodeEntry {
                parent,
                name: name.clone(),
                path,
                kind,
            },
        );

        self.lookup.insert((parent, name), ino);

        ino
    }
}

pub struct SharedInodeTable {
    inner: RwLock<InodeTable>,
}

impl SharedInodeTable {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(InodeTable::new()),
        }
    }

    pub fn get(&self, ino: u64) -> Option<InodeEntry> {
        self.inner
            .read()
            .unwrap()
            .get(ino)
            .cloned()
    }

    pub fn find(
        &self,
        parent: u64,
        name: &OsString,
    ) -> Option<u64> {
        self.inner
            .read()
            .unwrap()
            .find(parent, name)
    }

    pub fn insert(
        &self,
        parent: u64,
        name: OsString,
        path: PathBuf,
        kind: InodeKind,
    ) -> u64 {
        self.inner
            .write()
            .unwrap()
            .insert(parent, name, path, kind)
    }
}




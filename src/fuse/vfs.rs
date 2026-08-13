use std::ffi::OsStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualFile {
    Key,
    Pub,
}

impl VirtualFile {
    pub fn from_name(name: &OsStr) -> Option<Self> {
        match name.to_str()? {
            "key" => Some(Self::Key),
            "pub" => Some(Self::Pub),
            _ => None,
        }
    }
    pub fn is_name(name: &OsStr) -> bool {
        Self::from_name(name).is_some()
    }
}

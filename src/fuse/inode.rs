use fuser::INodeNo;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub const ROOT_INO: INodeNo = INodeNo(1);

pub fn inode_for_path(path: &Path) -> INodeNo {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.to_string_lossy().hash(&mut h);
    let n = h.finish();
    INodeNo(if n == 0 { 2 } else { n })
}

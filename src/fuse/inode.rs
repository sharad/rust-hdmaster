








pub const ROOT_INO: u64 = 1;

pub fn virtual_inode(parent: u64, name: &str) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut hasher =
        std::collections::hash_map::DefaultHasher::new();

    parent.hash(&mut hasher);
    name.hash(&mut hasher);

    let value = hasher.finish();

    if value == 0 {
        2
    } else {
        value
    }
}




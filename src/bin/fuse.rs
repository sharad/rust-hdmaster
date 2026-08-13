

use rust_hdmaster::fuse::{
    backing::LocalBackingStore,
    HdFuse,
};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    let backing =
        LocalBackingStore::new("/mnt/db/.hd-storage");

    let filesystem =
        HdFuse::new(backing, runtime);

    // FUSE mount here.

    Ok(())
}




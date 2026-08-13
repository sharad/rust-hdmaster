


use std::{
    env,
    path::PathBuf,
};

use fuser::MountOption;
use rust_hdmaster::fuse::{
    HdFuse,
    LocalBackingStore,
    PlaceholderVirtualFileProvider,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args =
        env::args_os().skip(1);

    let backing =
        PathBuf::from(
            args.next()
                .expect("missing backing directory"),
        );

    let mountpoint =
        PathBuf::from(
            args.next()
                .expect("missing mount point"),
        );

    let runtime =
        tokio::runtime::Runtime::new()?;

    let filesystem =
        HdFuse::new(
            LocalBackingStore::new(backing),
            PlaceholderVirtualFileProvider,
            runtime,
        );

    fuser::mount2(
        filesystem,
        mountpoint,
        &[
            MountOption::FSName(
                "hd-fuse".to_string(),
            ),
            MountOption::AutoUnmount,
        ],
    )?;

    Ok(())
}



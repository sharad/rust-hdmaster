


use clap::Parser;

use rust_hdmaster::fuse::{
    backing::LocalBackingStore,
    FillerGenerator,
    HdFuse,
};

use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    base: PathBuf,

    #[arg(long)]
    mount: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let backing = LocalBackingStore::new(args.base);

    let generator = FillerGenerator;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let filesystem = HdFuse::new(
        backing,
        generator,
        runtime,
    );

    let mut options = fuser::Config::default();

    options.mount_options = vec![
        fuser::MountOption::FSName("hdmaster".to_string()),
        fuser::MountOption::AutoUnmount,
    ];

    fuser::mount(
        filesystem,
        args.mount,
        &options,
    )?;

    Ok(())
}








use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_hdmaster::serialization::{save_node, write_private_pem, write_public_pem};
use rust_hdmaster::{DerivationPath, MasterSeed, NodeDeriver};
use rust_hdmaster::provider;
// use rust_hdmaster::provider::{Algorithm, };
// use rust_hdmaster::{DerivationScheme, HdNode};
use std::{path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(name = "hdmaster")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Seed {
        #[arg(short, long, default_value = "master.seed")]
        output: PathBuf,
    },
    #[cfg(feature = "mnemonic")]
    Mnemonic {
        #[arg(long, default_value = "")]
        passphrase: String,
    },
    DeriveSeed {
        #[arg(long)]
        seed_file: PathBuf,
        #[arg(long)]
        application: String,
        #[arg(long)]
        algorithm: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        path: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    DeriveHex {
        #[arg(long)]
        seed_hex: String,
        #[arg(long)]
        application: String,
        #[arg(long)]
        algorithm: String,
        #[arg(long)]
        scheme: String,
        #[arg(long)]
        path: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Child {
        #[arg(long)]
        parent_node: PathBuf,
        #[arg(long)]
        path: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}
fn emit_output(n: &rust_hdmaster::HdNode, o: Option<PathBuf>) -> Result<()> {
    if let Some(d) = o {
        std::fs::create_dir_all(&d)?;
        save_node(n, &d.join("node.json"))?;
        write_private_pem(n, &d.join("private.pem"))?;
        write_public_pem(n, &d.join("public.pem"))?;
        println!("{}", d.display())
    } else {
        println!(
            "algorithm={:?} scheme={:?} depth={} child={} public={} private=<hidden>",
            n.provider.algorithm,
            n.provider.scheme,
            n.depth,
            n.child_index,
            hex::encode(&n.public_key)
        );
    }
    Ok(())
}
fn main() -> Result<()> {
    let c = Cli::parse();
    let d = NodeDeriver::default();
    match c.command {
        Command::Seed { output } => MasterSeed::random_32().write_file(&output)?,
        #[cfg(feature = "mnemonic")]
        Command::Mnemonic { passphrase } => {
            use bip39::{Language, Mnemonic};
            let m = Mnemonic::generate_in(Language::English, 24)?;
            println!("mnemonic: {m}");
            let phrase = m.to_string();
            MasterSeed::from_mnemonic(&phrase, &passphrase)?
                .write_file(std::path::Path::new("master.seed"))?
        }
        Command::DeriveSeed {
            seed_file,
            application,
            algorithm,
            scheme,
            path,
            output,
        } => {
            let n = d.derive_from_seed(
                &MasterSeed::read_file(&seed_file)?,
                provider::Algorithm::from_str(&algorithm)?,
                provider::DerivationScheme::from_str(&scheme)?,
                &application,
                &DerivationPath::from_str(&path)?,
            )?;
            emit_output(&n, output)?
        }
        Command::DeriveHex {
            seed_hex,
            application,
            algorithm,
            scheme,
            path,
            output,
        } => {
            let n = d.derive_from_seed(
                &MasterSeed::from_hex(&seed_hex)?,
                provider::Algorithm::from_str(&algorithm)?,
                provider::DerivationScheme::from_str(&scheme)?,
                &application,
                &DerivationPath::from_str(&path)?,
            )?;
            emit_output(&n, output)?
        }
        Command::Child {
            parent_node,
            path,
            output,
        } => {
            let n =
                d.derive_child_from_node_file(&parent_node, &DerivationPath::from_str(&path)?)?;
            emit_output(&n, output)?
        }
    }
    Ok(())
}

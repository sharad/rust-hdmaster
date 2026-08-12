use hdmaster::serialization::save_node;
use hdmaster::{Algorithm, ChildIndex, DerivationPath, MasterSeed, NodeDeriver};
use std::{path::PathBuf, str::FromStr};
#[test]
fn memory_and_file_are_same() {
    let s = MasterSeed::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let d = NodeDeriver::default();
    let p = d
        .derive_from_seed(
            &s,
            Algorithm::Secp256k1,
            "crypto",
            &DerivationPath::from_str("44'/0'/0'").unwrap(),
        )
        .unwrap();
    let m = d
        .derive_child_from_node(&p, &DerivationPath::from_str("0/0/1").unwrap())
        .unwrap();
    let f = PathBuf::from("target/test-parent.json");
    save_node(&p, &f).unwrap();
    let x = d
        .derive_child_from_node_file(&f, &DerivationPath::from_str("0/0/1").unwrap())
        .unwrap();
    assert_eq!(m.private_key, x.private_key);
    assert_eq!(m.chain_code, x.chain_code);
    assert_eq!(m.public_key, x.public_key);
}
#[test]
fn secp_supports_non_hardened() {
    let s = MasterSeed::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let d = NodeDeriver::default();
    let n = d
        .derive_from_seed(
            &s,
            Algorithm::Secp256k1,
            "ssh",
            &DerivationPath::from_str("0'").unwrap(),
        )
        .unwrap();
    let c = d.derive_child(&n, ChildIndex::normal(7).unwrap()).unwrap();
    assert_eq!(c.depth, 2);
    assert_eq!(c.child_index, 7);
}
#[test]
fn deep_mixed_path() {
    let s = MasterSeed::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let d = NodeDeriver::default();
    let n = d
        .derive_from_seed(
            &s,
            Algorithm::Secp256k1,
            "crypto",
            &DerivationPath::from_str("44'/0'/0'/0/0/1/2'/3").unwrap(),
        )
        .unwrap();
    assert_eq!(n.depth, 8);
}
#[test]
fn ed25519_rejects_non_hardened() {
    let s = MasterSeed::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let d = NodeDeriver::default();
    assert!(d
        .derive_from_seed(
            &s,
            Algorithm::Ed25519,
            "ssh",
            &DerivationPath::from_str("0/1'").unwrap()
        )
        .is_err());
}
#[test]
fn parse_indexes() {
    assert!(ChildIndex::from_str("0'").unwrap().is_hardened());
    assert!(!ChildIndex::from_str("0").unwrap().is_hardened());
    assert_eq!(ChildIndex::from_str("17h").unwrap().number(), 17);
}

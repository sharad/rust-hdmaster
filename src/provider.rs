use hmac::{Hmac, Mac};
use sha2::Sha512;
use crate::{algorithm::Algorithm,error::{HdError,Result},node::{DerivationScheme,HdNode},path::ChildIndex};
type HmacSha512=Hmac<Sha512>;

pub trait Provider: Send+Sync { fn algorithm(&self)->Algorithm; fn scheme(&self)->DerivationScheme; fn supports_non_hardened(&self)->bool; fn master(&self,application:&str,seed:&[u8])->Result<HdNode>; fn child(&self,parent:&HdNode,index:ChildIndex)->Result<HdNode>; }
pub struct ProviderRegistry{ providers:Vec<Box<dyn Provider>> }
impl ProviderRegistry { pub fn standard()->Self{Self{providers:vec![Box::new(Secp),Box::new(Ed25519),Box::new(P256)]}} pub fn get(&self,a:Algorithm)->Result<&dyn Provider>{self.providers.iter().find(|p|p.algorithm()==a).map(|p|p.as_ref()).ok_or_else(||HdError::UnsupportedAlgorithm(format!("{a:?}")))}}
pub fn derive_child(p:&HdNode,i:ChildIndex)->Result<HdNode>{ProviderRegistry::standard().get(p.algorithm)?.child(p,i)}
fn mac(k:&[u8],d:&[u8])->Result<[u8;64]>{let mut m=HmacSha512::new_from_slice(k).map_err(|e|HdError::Crypto(e.to_string()))?;m.update(d);let o=m.finalize().into_bytes();Ok(o.into())}
fn edpub(k:&[u8])->Result<Vec<u8>>{use ed25519_dalek::{SigningKey,Verifier};let a:[u8;32]=k.try_into().map_err(|_|HdError::InvalidPrivateKey)?;Ok(SigningKey::from_bytes(&a).verifying_key().to_bytes().to_vec())}
fn ppub(k:&[u8])->Result<Vec<u8>>{use p256::{SecretKey,elliptic_curve::sec1::ToEncodedPoint};let s=SecretKey::from_slice(k).map_err(|_|HdError::InvalidPrivateKey)?;Ok(s.public_key().to_encoded_point(false).as_bytes().to_vec())}
fn spub(k:&[u8])->Result<Vec<u8>>{let s=secp256k1::Secp256k1::new();let sk=secp256k1::SecretKey::from_slice(k).map_err(|_|HdError::InvalidPrivateKey)?;Ok(secp256k1::PublicKey::from_secret_key(&s,&sk).serialize().to_vec())}

struct Secp;
impl Provider for Secp {
 fn algorithm(&self)->Algorithm{Algorithm::Secp256k1} fn scheme(&self)->DerivationScheme{DerivationScheme::Bip32} fn supports_non_hardened(&self)->bool{true}
 fn master(&self,a:&str,seed:&[u8])->Result<HdNode>{let i=mac(b"Bitcoin seed",seed)?;let sk=secp256k1::SecretKey::from_slice(&i[..32]).map_err(|_|HdError::InvalidPrivateKey)?;Ok(HdNode{application:a.into(),algorithm:Algorithm::Secp256k1,scheme:DerivationScheme::Bip32,depth:0,child_index:0,chain_code:i[32..].try_into().unwrap(),private_key:sk.secret_bytes().to_vec(),public_key:spub(&sk.secret_bytes())?})}
 fn child(&self,p:&HdNode,x:ChildIndex)->Result<HdNode>{let secp=secp256k1::Secp256k1::new();let parent=secp256k1::SecretKey::from_slice(&p.private_key).map_err(|_|HdError::InvalidPrivateKey)?;let mut d=Vec::new();if x.is_hardened(){d.push(0);d.extend(&p.private_key)}else{d.extend(&p.public_key)}d.extend(x.raw().to_be_bytes());let i=mac(&p.chain_code,&d)?;let tweak=secp256k1::Scalar::from_be_bytes(i[..32].try_into().unwrap()).map_err(|_|HdError::InvalidPrivateKey)?;let sk=parent.add_tweak(&tweak).map_err(|_|HdError::InvalidPrivateKey)?;Ok(HdNode{application:p.application.clone(),algorithm:p.algorithm,scheme:p.scheme,depth:p.depth+1,child_index:x.raw(),chain_code:i[32..].try_into().unwrap(),private_key:sk.secret_bytes().to_vec(),public_key:secp256k1::PublicKey::from_secret_key(&secp,&sk).serialize().to_vec()})}
}

struct Ed25519;
impl Provider for Ed25519 {
 fn algorithm(&self)->Algorithm{Algorithm::Ed25519} fn scheme(&self)->DerivationScheme{DerivationScheme::Slip10} fn supports_non_hardened(&self)->bool{false}
 fn master(&self,a:&str,seed:&[u8])->Result<HdNode>{let i=mac(b"ed25519 seed",seed)?;Ok(HdNode{application:a.into(),algorithm:Algorithm::Ed25519,scheme:DerivationScheme::Slip10,depth:0,child_index:0,chain_code:i[32..].try_into().unwrap(),private_key:i[..32].to_vec(),public_key:edpub(&i[..32])?})}
 fn child(&self,p:&HdNode,x:ChildIndex)->Result<HdNode>{if !x.is_hardened(){return Err(HdError::NonHardenedUnsupported)}let mut d=vec![0];d.extend(&p.private_key);d.extend(x.raw().to_be_bytes());let i=mac(&p.chain_code,&d)?;Ok(HdNode{application:p.application.clone(),algorithm:p.algorithm,scheme:p.scheme,depth:p.depth+1,child_index:x.raw(),chain_code:i[32..].try_into().unwrap(),private_key:i[..32].to_vec(),public_key:edpub(&i[..32])?})}
}

struct P256;
impl Provider for P256 {
 fn algorithm(&self)->Algorithm{Algorithm::P256} fn scheme(&self)->DerivationScheme{DerivationScheme::Slip10} fn supports_non_hardened(&self)->bool{false}
 fn master(&self,a:&str,seed:&[u8])->Result<HdNode>{let i=mac(b"Nist256p1 seed",seed)?;let sk=p256::SecretKey::from_slice(&i[..32]).map_err(|_|HdError::InvalidPrivateKey)?;Ok(HdNode{application:a.into(),algorithm:Algorithm::P256,scheme:DerivationScheme::Slip10,depth:0,child_index:0,chain_code:i[32..].try_into().unwrap(),private_key:sk.to_bytes().to_vec(),public_key:ppub(&sk.to_bytes())?})}
 fn child(&self,p:&HdNode,x:ChildIndex)->Result<HdNode>{if !x.is_hardened(){return Err(HdError::NonHardenedUnsupported)}let mut d=vec![0];d.extend(&p.private_key);d.extend(x.raw().to_be_bytes());let i=mac(&p.chain_code,&d)?;let sk=p256::SecretKey::from_slice(&i[..32]).map_err(|_|HdError::InvalidPrivateKey)?;Ok(HdNode{application:p.application.clone(),algorithm:p.algorithm,scheme:p.scheme,depth:p.depth+1,child_index:x.raw(),chain_code:i[32..].try_into().unwrap(),private_key:sk.to_bytes().to_vec(),public_key:ppub(&sk.to_bytes())?})}
}

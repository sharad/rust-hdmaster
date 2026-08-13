use thiserror::Error;

#[derive(Debug, Error)]
pub enum HdError {
    #[error("invalid derivation path: {0}")]
    InvalidPath(String),
    #[error("invalid child index: {0}")]
    InvalidChildIndex(String),
    #[error("non-hardened derivation is not supported by this provider")]
    NonHardenedUnsupported,
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("unsupported scheme: {0}")]
    UnsupportedDerivationScheme(String),
    #[error("invalid seed: {0}")]
    InvalidSeed(String),
    #[error("invalid private key")]
    InvalidPrivateKey,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("cryptographic error: {0}")]
    Crypto(String),
}
pub type Result<T> = std::result::Result<T, HdError>;

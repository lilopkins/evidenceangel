use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Package error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("The evidence package is corrupt ({0}).")]
    CorruptEvidencePackage(String),
    #[error("The package manifest is corrupt: {0}")]
    InvalidManifest(serde_json::Error),
    #[error("Failed to create package: {0}")]
    FailedToCreatePackage(serde_json::Error),
}

/// A result from EvidenceAngel.
pub type Result<T> = std::result::Result<T, Error>;

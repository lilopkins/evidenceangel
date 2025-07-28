use thiserror::Error;
use uuid::Uuid;

/// An error raised by `EvidenceAngel`.
#[derive(Debug, Error)]
pub enum Error {
    /// You are trying to perform an operation without a lock on the package.
    #[error(
        "The file you are working with is already open. Please close it and try again. If you are sure no one else if working with it, you can delete the lock file and try again."
    )]
    LockNotObtained,

    /// An I/O error from the system.
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    /// A package error, i.e. raised by the `zip` package.
    #[error("Package error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// The package is corrupt. See the contained string for more details.
    #[error("The evidence package is corrupt ({0}).")]
    CorruptEvidencePackage(String),

    /// The package manifest isn't valid JSON.
    #[error("The package manifest is corrupt: {0}")]
    InvalidManifest(serde_json::Error),

    /// The package couldn't be created due to a JSON error.
    #[error("Failed to create package: {0}")]
    FailedToCreatePackage(serde_json::Error),

    /// The test case couldn't be saved due to a JSON error.
    #[error("Failed to save test case: {0}")]
    FailedToSaveTestCase(serde_json::Error),

    /// The test case couldn't be read due to a JSON error.
    #[error("Failed to read test case {1}: {0}")]
    InvalidTestCase(serde_json::Error, Uuid),

    /// Some media is missing from the package.
    #[error("Media is missing from the package with hash {0}")]
    MediaMissing(String),

    /// Validation against the manifest schema failed.
    #[error(
        "Manifest schema validation failed. Perhaps this package is from a newer version of EvidenceAngel."
    )]
    ManifestSchemaValidationFailed,

    /// Validation against the test case schema failed.
    #[error(
        "Test case schema validation failed. Perhaps this package is from a newer version of EvidenceAngel."
    )]
    TestCaseSchemaValidationFailed,

    /// The specified test case doesn't exist
    #[error("The specified test case doesn't exist")]
    DoesntExist(Uuid),

    /// An otherwise unhandled error occured during export.
    #[error("Export failed: {0}")]
    OtherExportError(Box<dyn std::error::Error>),
}

/// A result from `EvidenceAngel`.
pub type Result<T> = std::result::Result<T, Error>;

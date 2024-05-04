use std::{
    fmt,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use getset::Getters;
use serde::{Deserialize, Serialize};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::{result::Error, Result};

/// An Evidence Package.
#[derive(Serialize, Deserialize, Getters)]
pub struct EvidencePackage {
    /// The internal ZIP file. This will never be `None`, as long as it has been correctly parsed.
    #[serde(skip)]
    zip: Option<ZipArchive<BufReader<File>>>,

    /// The metadata for the package.
    #[getset(get = "pub")]
    metadata: Metadata,
    media: Vec<MediaFile>,
    test_cases: Vec<TestCase>,
}

impl fmt::Debug for EvidencePackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvidencePackage")
            .field("metadata", &self.metadata)
            .field("media", &self.media)
            .field("test_cases", &self.test_cases)
            .finish()
    }
}

impl EvidencePackage {
    /// Create a new evidence package.
    pub fn new(path: PathBuf, title: String, authors: Vec<Author>) -> Result<Self> {
        // Create ZIP file
        let buf_writer = BufWriter::new(File::create_new(&path)?);
        let mut zip = ZipWriter::new(buf_writer);
        let options = SimpleFileOptions::default();

        // Create empty structure.
        zip.add_directory("media", options)?;
        zip.add_directory("testcases", options)?;

        // Create manifest data.
        let manifest = Self {
            zip: None,
            media: vec![],
            test_cases: vec![],
            metadata: Metadata {
                title,
                authors: authors.into(),
            },
        };
        let manifest_data =
            serde_json::to_string(&manifest).map_err(Error::FailedToCreatePackage)?;

        // Write ZIP file.
        zip.start_file("manifest.json", options)?;
        zip.write(manifest_data.as_bytes())?;
        zip.finish()?;

        Self::open(path)
    }

    /// Open an evidence package, returning either the parsed evidence package for manipulation, or an error.
    pub fn open(path: PathBuf) -> Result<Self> {
        // Open ZIP file
        let buf_reader = BufReader::new(File::open(path)?);
        let mut zip = ZipArchive::new(buf_reader)?;

        // Read manifest
        let manifest_entry = zip
            .by_name("manifest.json")
            .map_err(|_| Error::CorruptEvidencePackage("missing manifest".to_string()))?;
        let buf_manifest = BufReader::new(manifest_entry);

        // Parse manifest
        let mut evidence_package: EvidencePackage =
            serde_json::from_reader(buf_manifest).map_err(Error::InvalidManifest)?;
        evidence_package.zip = Some(zip);

        Ok(evidence_package)
    }
}

/// Evidence package metadata.
#[derive(Debug, Getters, Serialize, Deserialize)]
pub struct Metadata {
    /// The package title.
    #[getset(get = "pub")]
    title: String,

    /// The package authors.
    #[getset(get = "pub")]
    authors: Vec<Author>,
}

/// An author of an evidence package.
#[derive(Debug, Getters, Serialize, Deserialize)]
pub struct Author {
    /// The author's name.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    name: String,
    /// The author's email address.
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    email: Option<String>,
}

impl Author {
    /// Create a new author from a name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            email: None,
        }
    }

    /// Create a new author from a name and email address.
    pub fn new_with_email<S: Into<String>>(name: S, email_address: S) -> Self {
        Self {
            name: name.into(),
            email: Some(email_address.into()),
        }
    }
}

#[derive(Debug, Getters, Serialize, Deserialize)]
struct MediaFile {
    /// The SHA256 checksum of the media file.
    #[getset(get = "pub")]
    sha256_checksum: String,
    /// The MIME type of the media file.
    #[getset(get = "pub")]
    mime_type: String,
}

#[derive(Debug, Getters, Serialize, Deserialize)]
struct TestCase {
    /// A string to reference the test case internally. Usually a UUID.
    #[getset(get = "pub")]
    name: String,
}

use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::fmt;

/// [`EvidencePackage`](super::EvidencePackage) metadata.
#[derive(Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Metadata {
    /// The package title.
    pub(super) title: String,

    /// The package authors.
    #[get_mut = "pub"]
    pub(super) authors: Vec<Author>,
}

/// An author of an [`EvidencePackage`](super::EvidencePackage).
#[derive(Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Author {
    /// The author's name.
    name: String,
    /// The author's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.email.is_some() {
            write!(f, "{} <{}>", self.name, self.email.as_ref().unwrap())
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub(super) struct MediaFileManifestEntry {
    /// The SHA256 checksum of the media file.
    sha256_checksum: String,
    /// The MIME type of the media file.
    mime_type: String,
}

impl From<&crate::MediaFile> for MediaFileManifestEntry {
    fn from(value: &crate::MediaFile) -> Self {
        Self {
            sha256_checksum: value.hash(),
            mime_type: value
                .mime_type()
                .map(|t| t.mime_type())
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub(super) struct TestCaseManifestEntry {
    /// A string to reference the test case internally. Usually a UUID.
    name: Uuid,
}

impl TestCaseManifestEntry {
    pub(super) fn new(name: Uuid) -> Self {
        Self { name }
    }
}

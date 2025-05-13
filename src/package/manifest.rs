use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::{collections::HashMap, fmt};

/// [`EvidencePackage`](super::EvidencePackage) metadata.
#[derive(Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct Metadata {
    /// The package title.
    pub(super) title: String,

    /// The package description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) description: Option<String>,

    /// The package authors.
    #[get_mut = "pub"]
    pub(super) authors: Vec<Author>,

    /// Custom metadata fields for test cases
    #[get_mut = "pub"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) custom_test_case_metadata: Option<HashMap<String, CustomMetadataField>>,

    /// Extra fields that this implementation doesn't understand.
    #[get = "pub"]
    #[serde(flatten)]
    pub(super) extra_fields: HashMap<String, serde_json::Value>,
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

    /// Extra fields that this implementation doesn't understand.
    #[get = "pub"]
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

impl Author {
    /// Create a new author from a name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            email: None,
            extra_fields: HashMap::new(),
        }
    }

    /// Create a new author from a name and email address.
    pub fn new_with_email<S: Into<String>>(name: S, email_address: S) -> Self {
        Self {
            name: name.into(),
            email: Some(email_address.into()),
            extra_fields: HashMap::new(),
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

/// A custom metadata field for [`TestCase`](super::test_cases::TestCase)s.
#[derive(Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize, PartialEq, Eq)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CustomMetadataField {
    /// A user-friendly name for this custom property.
    name: String,
    /// A description for this custom property.
    description: String,
    /// Is this custom property the main one in this package?
    primary: bool,

    /// Extra fields that this implementation doesn't understand.
    #[get = "pub"]
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

/// The manifest entry for a media file present in the package.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub(super) struct MediaFileManifestEntry {
    /// The SHA256 checksum of the media file.
    sha256_checksum: String,
    /// The MIME type of the media file.
    mime_type: String,

    /// Extra fields that this implementation doesn't understand.
    #[get = "pub"]
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

impl From<&crate::MediaFile> for MediaFileManifestEntry {
    fn from(value: &crate::MediaFile) -> Self {
        Self {
            sha256_checksum: value.hash(),
            mime_type: value
                .mime_type()
                .map_or("unknown", |t| t.mime_type())
                .to_string(),
            extra_fields: HashMap::new(),
        }
    }
}

/// An entry in the manifest storing the [`Uuid`] for a test case present in the
/// package.
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub(super) struct TestCaseManifestEntry {
    /// A string to reference the test case internally. Usually a UUID.
    #[serde(alias = "name")] // Compatibility with previous pre-RFC field name `name`.
    id: Uuid,

    /// Extra fields that this implementation doesn't understand.
    #[get = "pub"]
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

impl TestCaseManifestEntry {
    /// Create a new test case manifest entry
    pub(super) fn new(id: Uuid) -> Self {
        Self {
            id,
            extra_fields: HashMap::new(),
        }
    }
}

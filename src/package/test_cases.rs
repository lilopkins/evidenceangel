use base64::Engine;
use chrono::{DateTime, Local};
use getset::{Getters, MutGetters, Setters};
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};
use uuid::Uuid;

const TESTCASE_SCHEMA_LOCATION: &str =
    "https://evidenceangel-schemas.hpkns.uk/manifest.1.schema.json";
pub(crate) const TESTCASE_SCHEMA: &str = include_str!("../../schemas/testcase.1.schema.json");

/// A test case stored within an [`EvidencePackage`](super::EvidencePackage).
#[derive(Clone, Debug, Serialize, Deserialize, Getters, MutGetters, Setters)]
pub struct TestCase {
    #[serde(rename = "$schema")]
    schema: String,

    /// The internal ID of this test case.
    #[serde(skip)]
    #[getset(get = "pub", set = "pub(super)")]
    id: Uuid,

    /// The metadata of this test case.
    #[getset(get = "pub", get_mut = "pub")]
    metadata: TestCaseMetadata,

    /// The evidence in this test case.
    #[getset(get = "pub", get_mut = "pub")]
    evidence: Vec<Evidence>,
}

impl TestCase {
    pub(super) fn new(id: Uuid, title: String, execution_datetime: DateTime<Local>) -> Self {
        Self {
            schema: TESTCASE_SCHEMA_LOCATION.to_string(),
            id,
            metadata: TestCaseMetadata {
                title,
                execution_datetime,
            },
            evidence: vec![],
        }
    }
}

/// The metadata of a [`TestCase`].
#[derive(Clone, Debug, Serialize, Deserialize, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct TestCaseMetadata {
    /// The title of the associated [`TestCase`].
    title: String,
    /// The time of execution of the associated [`TestCase`].
    execution_datetime: DateTime<Local>,
}

/// Evidence in a [`TestCase`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Getters, MutGetters, Setters)]
#[getset(get = "pub")]
pub struct Evidence {
    /// The kind of this evidence.
    kind: EvidenceKind,

    /// The data contained within this piece of evidence.
    #[getset(get_mut = "pub", set = "pub")]
    value: EvidenceData,

    /// A text caption associated with this piece of evidence.
    #[getset(get_mut = "pub", set = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
}

impl Evidence {
    /// Create a new evidence object.
    pub fn new(kind: EvidenceKind, value: EvidenceData) -> Self {
        Self {
            kind,
            value,
            caption: None,
        }
    }
}

/// Kinds of [`Evidence`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    /// A text entry.
    Text,
    /// An image.
    Image,
    /// An attached file.
    File,
    /// An HTTP request and response.
    Http,
}

/// Data in a piece of [`Evidence`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvidenceData {
    /// Text based data.
    Text {
        /// The text-based content.
        content: String,
    },
    /// Base64 encoded data.
    Base64 {
        /// The raw data which will be encoded as base64 automatically when saved.
        data: Vec<u8>,
    },
    /// A [`MediaFile`](crate::MediaFile). This is useful for large files that would be unreasonable to store as text or base64.
    Media {
        /// The hash of the [`MediaFile`](crate::MediaFile) that should be referred to. Note that you are responsible for adding
        /// a media file of the appropriate type to the package.
        hash: String,
    },
}

impl EvidenceData {
    /// Get the data from this object. This will fetch the media file if needed.
    pub fn get_data(&self, package: &mut crate::EvidencePackage) -> crate::Result<Vec<u8>> {
        match self {
            Self::Text { content } => Ok(content.clone().into_bytes()),
            Self::Base64 { data } => Ok(data.clone()),
            Self::Media { hash } => package
                .get_media(hash)?
                .map(|mf| mf.data().clone())
                .ok_or(crate::Error::MediaMissing(hash.clone())),
        }
    }
}

struct EvidenceDataVisitor;

impl<'de> Visitor<'de> for EvidenceDataVisitor {
    type Value = EvidenceData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(r#"evidence data, in the format: "kind:value""#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::visit_string(self, v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((typ, dat)) = v.split_once(':') {
            return match typ {
                "plain" => Ok(EvidenceData::Text {
                    content: dat.to_string(),
                }),
                "base64" => Ok(EvidenceData::Base64 {
                    data: base64::prelude::BASE64_STANDARD_NO_PAD
                        .decode(dat)
                        .map_err(serde::de::Error::custom)?,
                }),
                "media" => Ok(EvidenceData::Media {
                    hash: dat.to_string(),
                }),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid type {typ}, expected one of plain, base64 or media"
                ))),
            };
        }
        Err(de::Error::custom(
            "invalid format, expected string with `:' separator",
        ))
    }
}

impl Serialize for EvidenceData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data_s = match self {
            Self::Text { content } => format!("plain:{content}"),
            Self::Base64 { data } => format!(
                "base64:{}",
                base64::prelude::BASE64_STANDARD_NO_PAD.encode(data)
            ),
            Self::Media { hash } => format!("media:{hash}"),
        };
        serializer.serialize_str(&data_s)
    }
}

impl<'de> Deserialize<'de> for EvidenceData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(EvidenceDataVisitor)
    }
}

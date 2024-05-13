use getset::Getters;

/// A media file stored within an [`EvidencePackage`](super::EvidencePackage).
#[derive(Getters)]
pub struct MediaFile {
    /// The raw data of this media file.
    #[getset(get = "pub")]
    data: Vec<u8>,
}

impl MediaFile {
    /// Generate a SHA256 hash of this data.
    pub fn hash(&self) -> String {
        sha256::digest(&self.data)
    }

    /// Determine the MIME type of this data
    pub fn mime_type(&self) -> Option<infer::Type> {
        infer::get(&self.data)
    }
}

impl<D: Into<Vec<u8>>> From<D> for MediaFile {
    fn from(value: D) -> Self {
        Self { data: value.into() }
    }
}

use std::{
    collections::{
        hash_map::{Values, ValuesMut},
        HashMap,
    },
    fmt,
    io::{self, BufReader, Read, Write},
    path::PathBuf,
};

use chrono::{DateTime, FixedOffset, Local};
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};
use test_cases::{TESTCASE_SCHEMA, TESTCASE_SCHEMA_2};
use uuid::Uuid;
use zip::{result::ZipError, write::SimpleFileOptions};

use crate::{result::Error, zip_read_writer::ZipReaderWriter, Result};

/// Package manifests
mod manifest;
pub use manifest::*;

/// Media handling
mod media;
pub use media::MediaFile;

/// Test cases from packages
mod test_cases;
pub use test_cases::{Evidence, EvidenceData, EvidenceKind, TestCase, TestCaseMetadata};

/// The URL for $schema for manifest.json
const MANIFEST_SCHEMA_LOCATION: &str =
    "https://evidenceangel-schemas.hpkns.uk/manifest.1.schema.json";
/// The schema to validate manifest.json against
const MANIFEST_SCHEMA: &str = include_str!("../schemas/manifest.1.schema.json");

/// An Evidence Package.
#[derive(Serialize, Deserialize, Getters, MutGetters)]
pub struct EvidencePackage {
    /// The internal ZIP file. This will never be `None`, as long as it has been correctly parsed.
    #[serde(skip)]
    zip: ZipReaderWriter,
    /// The actual media data from this package
    #[serde(skip)]
    media_data: HashMap<String, MediaFile>,
    /// The actual test data from this package
    #[serde(skip)]
    test_case_data: HashMap<Uuid, TestCase>,

    /// The JSON schema for for this package
    #[serde(rename = "$schema")]
    schema: String,
    /// The metadata for the package.
    #[getset(get = "pub", get_mut = "pub")]
    metadata: Metadata,
    /// The manifest entries for the media in this package
    media: Vec<MediaFileManifestEntry>,
    /// The manifest entries for the test cases in this package
    test_cases: Vec<TestCaseManifestEntry>,
}

impl Clone for EvidencePackage {
    fn clone(&self) -> Self {
        Self {
            zip: self.zip.clone(),
            media_data: HashMap::new(),
            test_case_data: self.test_case_data.clone(),

            schema: MANIFEST_SCHEMA_LOCATION.to_string(),
            metadata: self.metadata.clone(),
            media: self.media.clone(),
            test_cases: self.test_cases.clone(),
        }
    }
}

impl fmt::Debug for EvidencePackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvidencePackage")
            .field("metadata", &self.metadata)
            .field("media", &self.media)
            .field("test_cases", &self.test_cases)
            .finish_non_exhaustive()
    }
}

impl EvidencePackage {
    /// Create a new evidence package.
    pub fn new(path: PathBuf, title: String, authors: Vec<Author>) -> Result<Self> {
        Self::new_with_description(path, title, None, authors)
    }

    /// Create a new evidence package with a specified description.
    pub fn new_with_description(
        path: PathBuf,
        title: String,
        description: Option<String>,
        authors: Vec<Author>,
    ) -> Result<Self> {
        // Create manifest data.
        let mut manifest = Self {
            zip: ZipReaderWriter::new(path),
            media_data: HashMap::new(),
            test_case_data: HashMap::new(),

            schema: MANIFEST_SCHEMA_LOCATION.to_string(),
            media: vec![],
            test_cases: vec![],
            metadata: Metadata {
                title,
                description,
                authors,
            },
        };
        let manifest_clone = manifest.clone_serde();

        // Create ZIP file
        let (_, zip) = manifest.zip.as_writer()?;
        let options = SimpleFileOptions::default();

        // Create empty structure.
        zip.add_directory("media", options)?;
        zip.add_directory("testcases", options)?;

        let manifest_data =
            serde_json::to_string(&manifest_clone).map_err(Error::FailedToCreatePackage)?;
        if !jsonschema::is_valid(
            &serde_json::from_str(MANIFEST_SCHEMA).expect("Schema is validated statically"),
            &serde_json::from_str(&manifest_data).expect("JSON just generated, shouldn't fail"),
        ) {
            return Err(Error::ManifestSchemaValidationFailed);
        }

        // Write ZIP file.
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest_data.as_bytes())?;
        manifest.zip.conclude_write()?;

        Ok(manifest)
    }

    /// Save the package to disk.
    pub fn save(&mut self) -> Result<()> {
        let mut clone = self.clone_serde();
        {
            // IMPORTANT!
            // This needs to be here to load the archive in read mode first, so that media can be migrated over.
            let _reader = self.zip.as_reader()?;
        }
        let (mut maybe_old_archive, zip) = self.zip.as_writer()?;
        let options = SimpleFileOptions::default();

        // Create empty structure.
        zip.add_directory("media", options)?;
        zip.add_directory("testcases", options)?;

        tracing::trace!("Current media cache: {:?}", self.media_data);

        let mut media_used = vec![];

        // Write any files as needed
        for test_case in &self.test_cases {
            let id = test_case.name();
            if let Some(data) = self.test_case_data.get(id) {
                // Whilst we are here, let's figure out what media we use.
                for evidence in data.evidence() {
                    if let EvidenceData::Media { hash } = evidence.value() {
                        media_used.push(hash);
                    }
                }

                let data = serde_json::to_string(data)
                    .map_err(crate::result::Error::FailedToSaveTestCase)?;
                if !jsonschema::is_valid(
                    &serde_json::from_str(TESTCASE_SCHEMA_2)
                        .expect("Schema is validated statically"),
                    &serde_json::from_str(&data).expect("JSON just generated, shouldn't fail"),
                ) {
                    let _ = self.zip.interrupt_write();
                    return Err(Error::TestCaseSchemaValidationFailed);
                }
                zip.start_file(format!("testcases/{id}.json"), options)?;
                zip.write_all(data.as_bytes())?;
            }
        }

        // Scrub unused media manifest entries
        self.media
            .retain(|entry| media_used.contains(&entry.sha256_checksum()));
        clone
            .media
            .retain(|entry| media_used.contains(&entry.sha256_checksum()));

        // Scrub media map of unreferenced entries
        self.media_data
            .retain(|hash, _val| media_used.contains(&hash));
        clone
            .media_data
            .retain(|hash, _val| media_used.contains(&hash));

        // Save media to package, either sourcing it from memory if present, or from the previous package.
        tracing::debug!("Media entries: {:?}", self.media);
        for entry in &self.media {
            let hash = entry.sha256_checksum();
            zip.start_file(format!("media/{hash}"), options)?;
            if self.media_data.contains_key(hash) {
                // If in memory, write from there
                tracing::trace!("Writing from cache {hash}");
                zip.write_all(self.media_data.get(hash).unwrap().data())?;
            } else {
                // Otherwise pull from previous package.
                // Consider moving this to not load entire file on move.
                if maybe_old_archive.is_some() {
                    tracing::debug!("Migrating media with hash {hash} from old file");
                    let old_archive = maybe_old_archive.as_mut().unwrap();
                    let res = old_archive.by_name(&format!("media/{hash}"));
                    match res {
                        Err(ZipError::FileNotFound) => {
                            return Err(Error::MediaMissing(hash.clone()))
                        }
                        Err(e) => {
                            tracing::error!("Error migrating from old package: {e}");
                            return Err(e.into());
                        }
                        Ok(mut file) => {
                            io::copy(&mut file, zip)?;
                        }
                    }
                } else {
                    unreachable!();
                }
            }
        }

        // Write manifest. This has to be done last to ensure media is scrubbed as needed.
        let manifest_data = serde_json::to_string(&clone).map_err(Error::FailedToCreatePackage)?;
        if !jsonschema::is_valid(
            &serde_json::from_str(MANIFEST_SCHEMA).expect("Schema is validated statically"),
            &serde_json::from_str(&manifest_data).expect("JSON just generated, shouldn't fail"),
        ) {
            let _ = self.zip.interrupt_write();
            return Err(Error::ManifestSchemaValidationFailed);
        }
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest_data.as_bytes())?;
        self.zip.conclude_write()?;
        Ok(())
    }

    /// Open an evidence package, returning either the parsed evidence package for manipulation, or an error.
    pub fn open(path: PathBuf) -> Result<Self> {
        // Open ZIP file
        let mut zip_rw = ZipReaderWriter::new(path);
        let zip = zip_rw.as_reader()?;

        // Read manifest
        let manifest_entry = zip
            .by_name("manifest.json")
            .map_err(|_| Error::CorruptEvidencePackage("missing manifest".to_string()))?;
        let manifest_data = {
            let mut buf_manifest = BufReader::new(manifest_entry);
            let mut manifest_data = String::new();
            buf_manifest.read_to_string(&mut manifest_data)?;
            manifest_data
        };

        // Validate manifest
        if !jsonschema::is_valid(
            &serde_json::from_str(MANIFEST_SCHEMA).expect("Schema is validated statically"),
            &serde_json::from_str(&manifest_data)
                .map_err(|_| Error::ManifestSchemaValidationFailed)?,
        ) {
            return Err(Error::TestCaseSchemaValidationFailed);
        }

        // Parse manifest
        let mut evidence_package: EvidencePackage =
            serde_json::from_str(&manifest_data).map_err(Error::InvalidManifest)?;

        // Read test cases
        for test_case in &evidence_package.test_cases {
            let id = test_case.name();
            let data = zip
                .by_name(&format!("testcases/{id}.json"))
                .map_err(|_| Error::CorruptEvidencePackage(format!("missing test case {id}")))?;
            let test_case_data = {
                let mut buf_test_case = BufReader::new(data);
                let mut test_case_data = String::new();
                buf_test_case.read_to_string(&mut test_case_data)?;
                test_case_data
            };

            // Validate test case against schema
            if jsonschema::is_valid(
                &serde_json::from_str(TESTCASE_SCHEMA_2).expect("Schema is validated statically"),
                &serde_json::from_str(&test_case_data)
                    .map_err(|_| Error::ManifestSchemaValidationFailed)?,
            ) {
                // Read as version 2
                let mut test_case: TestCase = serde_json::from_str(&test_case_data)
                    .map_err(|e| Error::InvalidTestCase(e, *id))?;
                test_case.set_id(*id);
                evidence_package.test_case_data.insert(*id, test_case);
            } else if jsonschema::is_valid(
                &serde_json::from_str(TESTCASE_SCHEMA).expect("Schema is validated statically"),
                &serde_json::from_str(&test_case_data)
                    .map_err(|_| Error::ManifestSchemaValidationFailed)?,
            ) {
                // Version 1 -> Version 2 migration
                // Load as normal, but set new schema URL
                let mut test_case: TestCase = serde_json::from_str(&test_case_data)
                    .map_err(|e| Error::InvalidTestCase(e, *id))?;
                test_case.set_id(*id);
                test_case.update_schema();
                evidence_package.test_case_data.insert(*id, test_case);
            } else {
                return Err(Error::TestCaseSchemaValidationFailed);
            }
        }

        evidence_package.zip = zip_rw;
        Ok(evidence_package)
    }

    /// Clone fields that will be serialized by serde
    fn clone_serde(&self) -> Self {
        Self {
            zip: ZipReaderWriter::new(PathBuf::new()),
            media_data: HashMap::new(),
            test_case_data: HashMap::new(),

            schema: MANIFEST_SCHEMA_LOCATION.to_string(),
            metadata: self.metadata.clone(),
            media: self.media.clone(),
            test_cases: self.test_cases.clone(),
        }
    }

    /// Obtain an iterator over test cases.
    /// Note that this is unsorted.
    pub fn test_case_iter(&self) -> Result<Values<Uuid, TestCase>> {
        Ok(self.test_case_data.values())
    }

    /// Obtain an iterator over test cases
    /// Note that this is unsorted.
    pub fn test_case_iter_mut(&mut self) -> Result<ValuesMut<Uuid, TestCase>> {
        Ok(self.test_case_data.values_mut())
    }

    /// Create a new test case.
    pub fn create_test_case<S>(&mut self, title: S) -> Result<&mut TestCase>
    where
        S: Into<String>,
    {
        self.create_test_case_at(title, Local::now().fixed_offset())
    }

    /// Create a new test case at a specified time.
    pub fn create_test_case_at<S>(
        &mut self,
        title: S,
        at: DateTime<FixedOffset>,
    ) -> Result<&mut TestCase>
    where
        S: Into<String>,
    {
        let new_id = uuid::Uuid::new_v4();

        // Create new manifest entry
        self.test_cases.push(TestCaseManifestEntry::new(new_id));

        // Create test case
        self.test_case_data
            .insert(new_id, TestCase::new(new_id, title.into(), at));

        Ok(self.test_case_data.get_mut(&new_id).unwrap())
    }

    /// Delete a test case.
    /// Returns true if a case was deleted.
    pub fn delete_test_case<U>(&mut self, id: U) -> Result<bool>
    where
        U: Into<Uuid>,
    {
        let id = id.into();

        // Search for matching case
        let index = self.test_cases.iter().position(|tc| *tc.name() == id);

        // Check a case was found
        if index.is_none() {
            return Ok(false);
        }

        // Remove case and data object
        let case = self.test_cases.remove(index.unwrap());
        self.test_case_data.remove(case.name());
        Ok(true)
    }

    /// Get a test case
    pub fn test_case<U>(&self, id: U) -> Result<Option<&TestCase>>
    where
        U: Into<Uuid>,
    {
        let id = id.into();

        // Search for matching case
        let case = self.test_cases.iter().find(|tc| *tc.name() == id);

        // Return case
        Ok(case.and_then(|tcme| self.test_case_data.get(tcme.name())))
    }

    /// Mutably get a test case
    pub fn test_case_mut<U>(&mut self, id: U) -> Result<Option<&mut TestCase>>
    where
        U: Into<Uuid>,
    {
        let id = id.into();

        // Search for matching case
        let case = self.test_cases.iter().find(|tc| *tc.name() == id);

        // Return case
        Ok(case.and_then(|tcme| self.test_case_data.get_mut(tcme.name())))
    }

    /// Add media to this package.
    ///
    /// Media is automatically removed if it is not referenced when [`EvidencePackage::save`] is called.
    /// As such, you can delete media simply by removing references to it.
    ///
    /// Media will remain in memory until [`EvidencePackage::save`] is called, at which point it will be
    /// written to disk. It will remain cached in memory until the [`EvidencePackage`] is dropped.
    pub fn add_media(&mut self, media_file: MediaFile) -> Result<&MediaFile> {
        let hash = media_file.hash();

        if !self
            .media
            .iter()
            .any(|entry| entry.sha256_checksum() == &hash)
        {
            // Create manifest entry
            let manifest_entry = MediaFileManifestEntry::from(&media_file);
            self.media.push(manifest_entry);

            // Insert data and return reference
            tracing::trace!("New media cache entry: {hash}");
            self.media_data.insert(hash.clone(), media_file);
        }

        Ok(self.get_media(&hash)?.unwrap())
    }

    /// Get media from this package by a sha256 hash.
    ///
    /// The in-memory cache will be searched first, then the file will be read again to pull the media.
    ///
    /// Returns [`None`] if the media couldn't be found with that hash.
    pub fn get_media<S>(&mut self, hash: S) -> Result<Option<&MediaFile>>
    where
        S: Into<String>,
    {
        let hash = hash.into();

        // Check in-memory cache
        if self.media_data.contains_key(&hash) {
            tracing::debug!("{hash} found in cache.");
            return Ok(self.media_data.get(&hash));
        }

        // Read from ZIP file
        let zip = self.zip.as_reader()?;
        let res = zip.by_name(&format!("media/{}", hash.clone()));
        match res {
            Ok(file) => {
                let size = file.size() as usize;
                let mut buf = Vec::with_capacity(size);
                tracing::debug!("Cache miss: {hash} (size: {size})");

                // Read from ZIP data
                let mut data = BufReader::new(file);
                data.read_to_end(&mut buf)?;

                // Add to in-memory cache
                let media: MediaFile = buf.into();
                tracing::trace!("New media cache entry: {hash}");
                self.media_data.insert(hash.clone(), media);

                // Return cached version
                Ok(Some(self.media_data.get(&hash).unwrap()))
            }
            Err(ZipError::FileNotFound) => {
                tracing::warn!("{hash} not found in package!");
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
}

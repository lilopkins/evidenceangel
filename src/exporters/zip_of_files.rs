use std::{
    collections::HashMap,
    fs,
    io::{self, BufWriter, Cursor},
};

use thiserror::Error;
use uuid::Uuid;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{EvidenceKind, EvidencePackage, TestCase};

use super::Exporter;

/// An exporter to an ZIP of the files in the package.
#[derive(Default)]
pub struct ZipOfFilesExporter;

/// An error from the [`ZipOfFilesExporter`].
#[derive(Debug, Error)]
pub enum ZipOfFilesError {
    /// No files are in this package, so no output would be produced
    #[error("no files are in this package, so no output would be produced")]
    NoFilesToExport,
}

impl Exporter for ZipOfFilesExporter {
    fn export_name() -> String {
        "ZIP Archive of Files".to_string()
    }

    fn export_extension() -> String {
        ".zip".to_string()
    }

    fn export_package(
        &mut self,
        package: &mut EvidencePackage,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        fn safely_add_cases_to_zip(
            mut zip: ZipWriter<BufWriter<fs::File>>,
            package: &mut EvidencePackage,
        ) -> crate::Result<()> {
            for test_case in package.test_case_iter()? {
                add_test_case_to_zip(&mut zip, package.clone(), test_case)
                    .map_err(crate::Error::OtherExportError)?;
            }

            zip.finish()
                .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;

            Ok(())
        }

        let mut has_files = false;
        for test_case in package.test_case_iter()? {
            if check_has_files(test_case) {
                has_files = true;
                break;
            }
        }
        if !has_files {
            return Err(crate::Error::OtherExportError(Box::new(
                ZipOfFilesError::NoFilesToExport,
            )));
        }

        let zip = ZipWriter::new(BufWriter::new(
            fs::File::create(&path).map_err(|e| crate::Error::OtherExportError(Box::new(e)))?,
        ));
        if let Err(e) = safely_add_cases_to_zip(zip, package) {
            // Delete file if exists
            let _ = fs::remove_file(path);

            return Err(e);
        }

        Ok(())
    }

    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        fn inner(
            mut zip: ZipWriter<BufWriter<fs::File>>,
            package: &mut EvidencePackage,
            case: &TestCase,
        ) -> crate::Result<()> {
            add_test_case_to_zip(&mut zip, package.clone(), case)
                .map_err(crate::Error::OtherExportError)?;

            zip.finish()
                .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;

            Ok(())
        }

        let case = package
            .test_case(case)?
            .ok_or(crate::Error::OtherExportError(
                "Test case not found!".into(),
            ))?
            .clone();

        if !check_has_files(&case) {
            return Err(crate::Error::OtherExportError(Box::new(
                ZipOfFilesError::NoFilesToExport,
            )));
        }

        let file =
            fs::File::create(&path).map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;
        let zip = ZipWriter::new(BufWriter::new(file));
        if let Err(e) = inner(zip, package, &case) {
            // Delete file if exists
            let _ = fs::remove_file(path);

            return Err(e);
        }

        Ok(())
    }
}

/// Check is this test case contains any file evidence
fn check_has_files(test_case: &TestCase) -> bool {
    for ev in test_case.evidence() {
        if let EvidenceKind::File = ev.kind() {
            return true;
        }
    }
    false
}

/// Create the worksheet that holds the test case's information
fn add_test_case_to_zip(
    zip: &mut ZipWriter<BufWriter<fs::File>>,
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("Creating ZIP of files for test case {}", test_case.id());

    let mut filename_count = HashMap::new();
    for ev in test_case.evidence() {
        if let EvidenceKind::File = ev.kind() {
            if let Some(filename) = ev.original_filename() {
                filename_count.insert(filename, filename_count.get(filename).unwrap_or(&0) + 1);
            }
        }
    }

    let mut unnamed_counter = 0;

    // Write evidence
    for evidence in test_case.evidence() {
        if let EvidenceKind::File = evidence.kind() {
            let data = evidence.value().get_data(&mut package)?;

            let name = if let Some(filename) = evidence.original_filename() {
                filename.clone()
            } else if let crate::EvidenceData::Media { hash } = evidence.value() {
                hash.clone()
            } else {
                unnamed_counter += 1;
                format!("unnamed-{unnamed_counter}")
            };

            let disambiguator = if let Some(count) = filename_count.get(&name) {
                if *count > 1 {
                    if let Some(caption) = evidence.caption() {
                        format!("({caption}) ")
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Add to ZIP file
            zip.start_file(
                format!("{}/{disambiguator}{name}", test_case.metadata().title()),
                SimpleFileOptions::default(),
            )
            .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;
            let mut data_cursor = Cursor::new(data);
            io::copy(&mut data_cursor, zip)
                .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;
        }
    }

    Ok(())
}

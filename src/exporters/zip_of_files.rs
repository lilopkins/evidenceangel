use std::{
    fs,
    io::{self, BufWriter, Cursor},
};

use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{EvidenceKind, EvidencePackage, TestCase};

use super::Exporter;

/// An exporter to an ZIP of the files in the package.
#[derive(Default)]
pub struct ZipOfFilesExporter;

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
        let mut zip = ZipWriter::new(BufWriter::new(
            fs::File::create(path).map_err(|e| crate::Error::OtherExportError(Box::new(e)))?,
        ));

        for test_case in package.test_case_iter()? {
            add_test_case_to_zip(&mut zip, package.clone(), test_case)
                .map_err(crate::Error::OtherExportError)?;
        }

        zip.finish()
            .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;

        Ok(())
    }

    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut zip = ZipWriter::new(BufWriter::new(
            fs::File::create(path).map_err(|e| crate::Error::OtherExportError(Box::new(e)))?,
        ));
        let case = package
            .test_case(case)?
            .ok_or(crate::Error::OtherExportError(
                "Test case not found!".into(),
            ))?;

        add_test_case_to_zip(&mut zip, package.clone(), case)
            .map_err(crate::Error::OtherExportError)?;

        zip.finish()
            .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;

        Ok(())
    }
}

/// Create the worksheet that holds the test case's information
fn add_test_case_to_zip(
    zip: &mut ZipWriter<BufWriter<fs::File>>,
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Creating ZIP of files for test case {}", test_case.id());

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

            // Add to ZIP file
            zip.start_file(name, SimpleFileOptions::default())
                .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;
            let mut data_cursor = Cursor::new(data);
            io::copy(&mut data_cursor, zip)
                .map_err(|e| crate::Error::OtherExportError(Box::new(e)))?;
        }
    }

    Ok(())
}

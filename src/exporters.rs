use std::path::PathBuf;

use uuid::Uuid;

use crate::{EvidencePackage, Result};

/// Exporter for Excel files.
#[cfg(feature = "exporter-excel")]
pub mod excel;
/// Exporter for HTML.
#[cfg(feature = "exporter-html")]
pub mod html;
/// Exporter for a ZIP of the files.
#[cfg(feature = "exporter-zip-of-files")]
pub mod zip_of_files;

/// Exporters can take an `EvidencePackage` and a target file path and export to other formats.
pub trait Exporter {
    /// The name of this exporter.
    fn export_name() -> String;
    /// The file extension to suggest when saving this file.
    fn export_extension() -> String;

    /// Export a package.
    ///
    /// # Errors
    ///
    /// Returns an error if the export failed for any reason.
    fn export_package(&mut self, package: &mut EvidencePackage, path: PathBuf) -> Result<()>;
    /// Export a test case.
    ///
    /// # Errors
    ///
    /// Returns an error if the export failed for any reason.
    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: PathBuf,
    ) -> Result<()>;
}

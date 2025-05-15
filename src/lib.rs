#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

//! # `EvidenceAngel`
//!
//! `EvidenceAngel` is a new tool in the Angel-suite to collect test evidence
//! from both manual and automated testing.

/// Locking file
mod lock_file;
/// The types of data in a package
mod package;
pub use package::{
    Author, CustomMetadataField, Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile,
    Metadata, TestCase, TestCaseMetadata, TestCasePassStatus,
};
/// The results of this crate
mod result;
pub use result::{Error, Result};
/// Exporters allow packages and test cases to be exported to different file formats.
pub mod exporters;
/// Open a ZIP file in a fashion that allows it to be switched between reading and writing.
mod zip_read_writer;

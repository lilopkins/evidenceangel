#![warn(missing_docs)]

//! # EvidenceAngel
//!
//! EvidenceAngel is a new tool in the Angel-suite to collect test evidence
//! from both manual and automated testing.

mod package;
pub use package::{
    Author, Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile, Metadata, TestCase,
    TestCaseMetadata,
};
mod result;
pub use result::{Error, Result};
mod zip_read_writer;

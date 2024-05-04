#![warn(missing_docs)]

//! # EvidenceAngel
//!
//! EvidenceAngel is a new tool in the Angel-suite to collect test evidence
//! from both manual and automated testing.

mod package;
mod result;
pub use package::*;
pub use result::Result;

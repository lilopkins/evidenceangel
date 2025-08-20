use std::fmt;

use colored::Colorize;
use schemars::{JsonSchema, schema_for};
use serde::Serialize;

use crate::{export::CliExportResult, package::CliPackage, test_cases::CliTestCase};

use super::error::{CliError, CliErrorContainer};

/// A serializable result from the EvidenceAngel CLI tool.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum CliData {
    /// Success happened, with no further data to display.
    Success,
    /// An error is returned, with the error specified by the `error` and `message` fields.
    Error(CliErrorContainer),
    /// A package is returned in full.
    Package(CliPackage),
    /// A test case is returned in full.
    TestCase(CliTestCase),
    /// A result of an export job.
    ExportResult(CliExportResult),
}

impl CliData {
    /// Return the JSON schema for the [`CliResult`] struct.
    pub fn schema() -> String {
        serde_json::to_string_pretty(&schema_for!(CliData)).unwrap()
    }
}

impl fmt::Display for CliData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliData::Success => write!(f, "{}", "Succeeded.".bold().green()),
            CliData::Error(e) => write!(f, "{} {}", "error:".bold().red(), e.message()),
            CliData::Package(p) => p.fmt(f),
            CliData::TestCase(t) => t.fmt(f),
            CliData::ExportResult(e) => e.fmt(f),
        }
    }
}

impl From<CliError> for CliData {
    fn from(error: CliError) -> Self {
        Self::Error(CliErrorContainer::from(error))
    }
}

use std::rc::Rc;

use getset::Getters;
use schemars::JsonSchema;
use serde::Serialize;
use thiserror::Error;

/// An error from the CLI tool
#[derive(Error, Debug, Clone)]
pub enum CliError {
    /// --file is mandated for all subcommands except a `shell-completions` and `json-schema`
    #[error(
        "all subcommands except `shell-completions` and `json-schema` require --file to be specified"
    )]
    MissingFile,

    /// the data you requested cannot be serialized as JSON
    #[error("the data you requested cannot be serialized as JSON")]
    CannotBeSerialized,

    /// failed to save the evidence package
    #[error("failed to save package: {0}")]
    FailedToSavePackage(Rc<evidenceangel::Error>),

    /// failed to read the evidence package
    #[error("failed to read package: {0}")]
    FailedToReadPackage(Rc<evidenceangel::Error>),

    /// invalid export format specified
    #[error("invalid export format `{0}`")]
    InvalidExportFormat(String),

    /// failed to export to file
    #[error("failed to export: {0}")]
    FailedToExport(Rc<evidenceangel::Error>),

    /// the provided string is not a one-based index and does not match a single test case exclusively
    #[error(
        "the value `{0}` is not a one-based index of a test case and does not exclusively match one test case"
    )]
    CannotMatchTestCase(String),

    /// the provided value does not match a one-based index of some evidence
    #[error("the value `{0}` does not match a one-based index of some evidence")]
    CannotMatchEvidence(usize),

    /// the provided date and time could not be parsed
    #[error("the provided date and time could not be parsed")]
    InvalidExecutionDateTime,

    /// failed to read a file you tried to load in
    #[error("failed to read a file you tried to load in")]
    FailedToReadFile,

    /// invalid image provided
    #[error("invalid image provided")]
    InvalidImage,

    /// couldn't add media to package
    #[error("couldn't add media to package")]
    CouldntAddMedia,
}

/// Get the name of the [`CliError`] variant, without any args, as a [`String`].
fn get_error_name(error: &CliError) -> &'static str {
    match error {
        CliError::MissingFile => "MissingFile",
        CliError::CannotBeSerialized => "CannotBeSerialized",
        CliError::FailedToSavePackage(_) => "FailedToCreatePackage",
        CliError::FailedToReadPackage(_) => "FailedToReadPackage",
        CliError::InvalidExportFormat(_) => "InvalidExportFormat",
        CliError::FailedToExport(_) => "FailedToExport",
        CliError::CannotMatchTestCase(_) => "CannotMatchTestCase",
        CliError::CannotMatchEvidence(_) => "CannotMatchEvidence",
        CliError::InvalidExecutionDateTime => "InvalidExecutionDateTime",
        CliError::FailedToReadFile => "FailedToReadFile",
        CliError::InvalidImage => "InvalidImage",
        CliError::CouldntAddMedia => "CouldntAddMedia",
    }
}

/// Contain a [`CliError`] in a serializable format compatible with JSON output.
#[derive(Serialize, JsonSchema, Getters)]
pub struct CliErrorContainer {
    /// A fixed identifier for this particular kind of error.
    error: &'static str,
    /// A human-readable message describing this error.
    #[get = "pub(crate)"]
    message: String,
}

impl From<CliError> for CliErrorContainer {
    fn from(error: CliError) -> Self {
        CliErrorContainer {
            error: get_error_name(&error),
            message: error.to_string(),
        }
    }
}

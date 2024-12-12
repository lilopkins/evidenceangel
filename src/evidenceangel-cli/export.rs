use std::{fmt, path::PathBuf, rc::Rc};

use clap::Subcommand;
use evidenceangel::{
    exporters::{excel::ExcelExporter, html::HtmlExporter, Exporter},
    EvidencePackage,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::result::{CliData, CliError};

/// Subcommands to work on packages
#[derive(Subcommand, Clone)]
pub enum ExportSubcommand {
    /// Export to another format.
    Package {
        /// The format to export to. Permitted values are "html" and "excel".
        #[arg(index = 1)]
        format: String,
        /// The target file to write.
        #[arg(index = 2)]
        target: PathBuf,
    },

    /// Export a test case to another format.
    TestCase {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The format to export to. Permitted values are "html" and "excel".
        #[arg(index = 2)]
        format: String,
        /// The target file to write.
        #[arg(index = 3)]
        target: PathBuf,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct CliExportResult {
    /// The format that has been exported to
    format: ExportFormat,
    /// The amount of data exported
    scope: ExportScope,
    /// The path that has been exported to
    path: PathBuf,
}

/// The export format
#[derive(Serialize, JsonSchema)]
enum ExportFormat {
    /// The export was to an Excel workbook
    Excel,
    /// The export was to HTML
    Html,
}

/// The scope of data exported
#[derive(Serialize, JsonSchema)]
enum ExportScope {
    /// An entire package was exported
    Package,
    /// A test case was exported
    TestCase {
        /// The title of the exported test case
        title: String,
    },
}

impl fmt::Display for CliExportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Exported {} to {}",
            match self.format {
                ExportFormat::Excel => "Excel",
                ExportFormat::Html => "HTML",
            },
            self.path.display()
        )
    }
}

/// Process the export subcommand
pub fn process(path: PathBuf, command: &ExportSubcommand) -> CliData {
    match command {
        ExportSubcommand::Package { format, target } => match EvidencePackage::open(path) {
            Ok(mut package) => match format.to_ascii_lowercase().as_str() {
                "excel" => {
                    let mut exporter = ExcelExporter;
                    if let Err(e) = exporter.export_package(&mut package, target.clone()) {
                        return CliError::FailedToExport(Rc::new(e)).into();
                    }

                    CliData::ExportResult(CliExportResult {
                        format: ExportFormat::Excel,
                        scope: ExportScope::Package,
                        path: target.clone(),
                    })
                }
                "html" => {
                    let mut exporter = HtmlExporter;
                    if let Err(e) = exporter.export_package(&mut package, target.clone()) {
                        return CliError::FailedToExport(Rc::new(e)).into();
                    }

                    CliData::ExportResult(CliExportResult {
                        format: ExportFormat::Html,
                        scope: ExportScope::Package,
                        path: target.clone(),
                    })
                }
                _ => CliError::InvalidExportFormat(format.clone()).into(),
            },
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },

        ExportSubcommand::TestCase {
            case,
            format,
            target,
        } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                // match against a test case
                let mut test_cases: Vec<_> = package
                    .test_case_iter()
                    .unwrap()
                    .map(|tc| {
                        (
                            *tc.id(),
                            tc.metadata().title().clone(),
                            *tc.metadata().execution_datetime(),
                        )
                    })
                    .collect();
                test_cases.sort_by(|(_, _, a), (_, _, b)| a.cmp(b));

                let case_id = match case.parse::<usize>() {
                    Ok(idx) => {
                        if idx == 0 || idx > test_cases.len() {
                            None
                        } else {
                            let idx = idx - 1;
                            Some((test_cases[idx].0, test_cases[idx].1.clone()))
                        }
                    }
                    Err(_) => {
                        // Try to match substring
                        let maybe_result: Vec<_> = test_cases
                            .iter()
                            .filter(|(_, title, _)| {
                                title
                                    .to_ascii_lowercase()
                                    .contains(&case.to_ascii_lowercase())
                            })
                            .collect();
                        if maybe_result.len() == 1 {
                            Some((maybe_result[0].0, maybe_result[0].1.clone()))
                        } else {
                            None
                        }
                    }
                };
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let (case_id, case_title) = case_id.unwrap();

                match format.to_ascii_lowercase().as_str() {
                    "excel" => {
                        let mut exporter = ExcelExporter;
                        if let Err(e) = exporter.export_case(&mut package, case_id, target.clone())
                        {
                            return CliError::FailedToExport(Rc::new(e)).into();
                        }

                        CliData::ExportResult(CliExportResult {
                            format: ExportFormat::Excel,
                            scope: ExportScope::TestCase { title: case_title },
                            path: target.clone(),
                        })
                    }
                    "html" => {
                        let mut exporter = HtmlExporter;
                        if let Err(e) = exporter.export_case(&mut package, case_id, target.clone())
                        {
                            return CliError::FailedToExport(Rc::new(e)).into();
                        }

                        CliData::ExportResult(CliExportResult {
                            format: ExportFormat::Html,
                            scope: ExportScope::TestCase { title: case_title },
                            path: target.clone(),
                        })
                    }
                    _ => CliError::InvalidExportFormat(format.clone()).into(),
                }
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
    }
}

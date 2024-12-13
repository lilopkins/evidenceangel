use std::{
    fmt::{self, Write},
    fs,
    io::{self, Cursor, Read},
    path::PathBuf,
    rc::Rc,
};

use chrono::FixedOffset;
use clap::Subcommand;
use colored::Colorize;
use evidenceangel::{Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::result::{CliData, CliError};

/// Subcommands to work on test cases
#[derive(Subcommand, Clone)]
pub enum TestCasesSubcommand {
    /// Create a new test case.
    Create {
        /// The title of the new test case.
        #[arg(index = 1)]
        title: String,
        /// The execution time of the new test case.
        #[arg(short, long)]
        executed_at: Option<String>,
    },
    /// View a test case.
    Read {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
    },
    /// Update a test case.
    Update {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The new title of the test case.
        #[arg(short, long)]
        title: Option<String>,
        /// The new execution time of the test case.
        #[arg(short, long)]
        executed_at: Option<String>,
    },
    /// Delete a test case from a package.
    Delete {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
    },
    /// Add evidence to a test case.
    AddEvidence {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The evidence to add
        #[command(subcommand)]
        evidence_value: EvidenceValue,
    },
    /// Read some evidence to stdout
    ReadEvidence {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The index of the evidence to read.
        #[arg(index = 2)]
        evidence_id: usize,
    },
    /// Delete evidence from a test case.
    UpdateEvidence {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The index of the evidence to delete.
        #[arg(index = 2)]
        evidence_id: usize,
        /// The new evidence
        #[command(subcommand)]
        evidence_value: EvidenceValue,
    },
    /// Delete evidence from a test case.
    DeleteEvidence {
        /// The one-based index of the test case to delete, or enough of the title to uniquely match against one test case.
        #[arg(index = 1)]
        case: String,
        /// The index of the evidence to delete.
        #[arg(index = 2)]
        evidence_id: usize,
    },
}

#[derive(Subcommand, Clone)]
pub enum EvidenceValue {
    /// Text-based evidence
    Text {
        /// The text to add, or `-` to read from stdin.
        #[arg(index = 1, default_value = "-")]
        value: String,
    },
    /// Image-based evidence
    Image {
        /// The image to add as evidence
        #[arg(index = 1)]
        image: PathBuf,
        /// An optional caption
        #[arg(index = 2)]
        caption: Option<String>,
    },
    /// An HTTP request/response
    Http {
        /// The text to add, or `-` to read from stdin. You can use a record separator (0x1e) to separate the request and response sections.
        #[arg(index = 1, default_value = "-")]
        value: String,
        /// An optional caption
        #[arg(index = 2)]
        caption: Option<String>,
    },
    /// A file
    File {
        /// The file to add as evidence.
        #[arg(index = 1)]
        path: PathBuf,
        /// An optional caption
        #[arg(index = 2)]
        caption: Option<String>,
    },
}

/// The data of a test case
#[derive(Serialize, JsonSchema)]
pub struct CliTestCase {
    /// The name of the test case
    name: String,
    /// The time the test case was executed
    executed_at: chrono::DateTime<FixedOffset>,
    /// The evidence in the test case
    evidence: Vec<CliEvidence>,
}

impl fmt::Display for CliTestCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🧪 {}", self.name.bold())?;
        writeln!(f, "  {}\n", self.executed_at.to_string().magenta())?;

        for (idx, ev) in self.evidence.iter().enumerate() {
            writeln!(
                f,
                "{}\n{}",
                format!("[Evidence #{}]", idx + 1).blue(),
                match ev {
                    CliEvidence::Text { data } => data
                        .clone()
                        .lines()
                        .fold(String::new(), |mut output, l| {
                            let _ = writeln!(output, "> {l}");
                            output
                        })
                        .trim_end()
                        .to_string(),
                    CliEvidence::Http => "HTTP request".magenta().to_string(),
                    CliEvidence::Image => "Image".magenta().to_string(),
                    CliEvidence::File { original_filename } => format!(
                        "{} {}",
                        "File".magenta(),
                        original_filename.clone().unwrap_or_default()
                    ),
                }
            )?;
        }

        Ok(())
    }
}

/// The data of a test case
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum CliEvidence {
    /// Text based evidence
    Text {
        /// The data in this textual evidence
        data: String,
    },
    /// An HTTP request and response
    Http,
    /// An image
    Image,
    /// File evidence
    File {
        /// The original filename of this evidence, if available.
        original_filename: Option<String>,
    },
}

/// Match a test case by a string, either a number (id) of the test case, or a
/// partial text match to the title
fn match_test_case(package: &mut EvidencePackage, case: String) -> Option<Uuid> {
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

    match case.parse::<usize>() {
        Ok(idx) => {
            if idx == 0 || idx > test_cases.len() {
                None
            } else {
                let idx = idx - 1;
                Some(test_cases[idx].0)
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
                Some(maybe_result[0].0)
            } else {
                None
            }
        }
    }
}

/// Convert an [`EvidenceValue`] from the CLI args to an [`Evidence`] struct
/// from EvidenceAngel.
fn evidence_from_evidence_value(
    evidence_value: EvidenceValue,
    package: &mut EvidencePackage,
) -> Result<Evidence, CliError> {
    match evidence_value.clone() {
        EvidenceValue::Text { mut value } => {
            let mut buf = vec![];
            if value == "-" {
                io::stdin()
                    .read_to_end(&mut buf)
                    .expect("failed to read stdin");
                value = String::from_utf8_lossy(&buf).into_owned();
            }
            Ok(Evidence::new(
                EvidenceKind::Text,
                EvidenceData::Text { content: value },
            ))
        }
        EvidenceValue::Image { image, caption } => {
            let media: MediaFile = fs::read(image)
                .map_err(|_| CliError::FailedToReadFile)?
                .into();
            let hash = media.hash();
            if let Some(mime) = media.mime_type() {
                if !["image/png", "image/jpeg"].contains(&mime.mime_type()) {
                    return Err(CliError::InvalidImage);
                }
            } else {
                return Err(CliError::InvalidImage);
            }
            package
                .add_media(media)
                .map_err(|_| CliError::CouldntAddMedia)?;
            let mut evidence = Evidence::new(EvidenceKind::Image, EvidenceData::Media { hash });
            evidence.set_caption(caption.clone());
            Ok(evidence)
        }
        EvidenceValue::Http { value, caption } => {
            let mut buf = vec![];
            if value == "-" {
                io::stdin()
                    .read_to_end(&mut buf)
                    .expect("failed to read stdin");
            } else {
                buf = value.into_bytes();
            }
            let mut evidence =
                Evidence::new(EvidenceKind::Http, EvidenceData::Base64 { data: buf });
            evidence.set_caption(caption.clone());
            Ok(evidence)
        }
        EvidenceValue::File { path, caption } => {
            let media: MediaFile = fs::read(&path)
                .map_err(|_| CliError::FailedToReadFile)?
                .into();
            let hash = media.hash();
            package
                .add_media(media)
                .map_err(|_| CliError::CouldntAddMedia)?;
            let mut evidence = Evidence::new(EvidenceKind::File, EvidenceData::Media { hash });
            evidence.set_caption(caption.clone());
            evidence
                .set_original_filename(path.file_name().map(|s| s.to_string_lossy().to_string()));
            Ok(evidence)
        }
    }
}

/// Process the test-cases subcommand
pub fn process(path: PathBuf, command: &TestCasesSubcommand) -> CliData {
    match command {
        TestCasesSubcommand::Create { title, executed_at } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = {
                    if let Some(executed_at) = executed_at {
                        match parse_datetime::parse_datetime(executed_at) {
                            Ok(dt) => {
                                Ok(*package.create_test_case_at(title.clone(), dt).unwrap().id())
                            }
                            Err(_) => Err(CliError::InvalidExecutionDateTime),
                        }
                    } else {
                        Ok(*package.create_test_case(title.clone()).unwrap().id())
                    }
                };
                if let Err(e) = &case_id {
                    return e.clone().into();
                }
                let case_id = case_id.unwrap();

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                CliData::TestCase(CliTestCase {
                    name: test_case.metadata().title().clone(),
                    executed_at: *test_case.metadata().execution_datetime(),
                    evidence: test_case
                        .evidence()
                        .iter()
                        .map(|ev| match ev.kind() {
                            EvidenceKind::Text => CliEvidence::Text {
                                data: String::from_utf8(ev.value().get_data(&mut package).unwrap())
                                    .unwrap(),
                            },
                            EvidenceKind::Image => CliEvidence::Image,
                            EvidenceKind::Http => CliEvidence::Http,
                            EvidenceKind::File => CliEvidence::File {
                                original_filename: ev.original_filename().clone(),
                            },
                        })
                        .collect(),
                })
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::Read { case } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = match_test_case(&mut package, case.clone());
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let case_id = case_id.unwrap();
                let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                CliData::TestCase(CliTestCase {
                    name: test_case.metadata().title().clone(),
                    executed_at: *test_case.metadata().execution_datetime(),
                    evidence: test_case
                        .evidence()
                        .iter()
                        .map(|ev| match ev.kind() {
                            EvidenceKind::Text => CliEvidence::Text {
                                data: String::from_utf8(ev.value().get_data(&mut package).unwrap())
                                    .unwrap(),
                            },
                            EvidenceKind::Image => CliEvidence::Image,
                            EvidenceKind::Http => CliEvidence::Http,
                            EvidenceKind::File => CliEvidence::File {
                                original_filename: ev.original_filename().clone(),
                            },
                        })
                        .collect(),
                })
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::Update {
            case,
            title,
            executed_at,
        } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = match_test_case(&mut package, case.clone());
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let case_id = case_id.unwrap();

                {
                    let test_case = package.test_case_mut(case_id).unwrap().unwrap();
                    if let Some(title) = title {
                        test_case.metadata_mut().set_title(title.clone());
                    }
                    if let Some(executed_at) = executed_at {
                        match parse_datetime::parse_datetime(executed_at) {
                            Ok(dt) => {
                                test_case.metadata_mut().set_execution_datetime(dt);
                            }
                            Err(_) => return CliError::InvalidExecutionDateTime.into(),
                        }
                    }
                }
                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                CliData::TestCase(CliTestCase {
                    name: test_case.metadata().title().clone(),
                    executed_at: *test_case.metadata().execution_datetime(),
                    evidence: test_case
                        .evidence()
                        .iter()
                        .map(|ev| match ev.kind() {
                            EvidenceKind::Text => CliEvidence::Text {
                                data: String::from_utf8(ev.value().get_data(&mut package).unwrap())
                                    .unwrap(),
                            },
                            EvidenceKind::Image => CliEvidence::Image,
                            EvidenceKind::Http => CliEvidence::Http,
                            EvidenceKind::File => CliEvidence::File {
                                original_filename: ev.original_filename().clone(),
                            },
                        })
                        .collect(),
                })
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::Delete { case } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = match_test_case(&mut package, case.clone());
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let case_id = case_id.unwrap();
                package.delete_test_case(case_id).unwrap();

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }
                CliData::Success
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::AddEvidence {
            case,
            evidence_value,
        } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = match_test_case(&mut package, case.clone());
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let case_id = case_id.unwrap();

                match evidence_from_evidence_value(evidence_value.clone(), &mut package) {
                    Ok(ev) => {
                        let test_case = package.test_case_mut(case_id).unwrap().unwrap();
                        test_case.evidence_mut().push(ev);
                    }
                    Err(e) => return e.into(),
                }

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                CliData::TestCase(CliTestCase {
                    name: test_case.metadata().title().clone(),
                    executed_at: *test_case.metadata().execution_datetime(),
                    evidence: test_case
                        .evidence()
                        .iter()
                        .map(|ev| match ev.kind() {
                            EvidenceKind::Text => CliEvidence::Text {
                                data: String::from_utf8(ev.value().get_data(&mut package).unwrap())
                                    .unwrap(),
                            },
                            EvidenceKind::Image => CliEvidence::Image,
                            EvidenceKind::Http => CliEvidence::Http,
                            EvidenceKind::File => CliEvidence::File {
                                original_filename: ev.original_filename().clone(),
                            },
                        })
                        .collect(),
                })
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::ReadEvidence { case, evidence_id } => {
            match EvidencePackage::open(path) {
                Ok(mut package) => {
                    let case_id = match_test_case(&mut package, case.clone());
                    if case_id.is_none() {
                        return CliError::CannotMatchTestCase(case.clone()).into();
                    }
                    let case_id = case_id.unwrap();

                    let test_case = package.test_case(case_id).unwrap().unwrap();
                    if *evidence_id < 1 || *evidence_id > test_case.evidence().len() {
                        return CliError::CannotMatchEvidence(*evidence_id).into();
                    }
                    let evidence_id = *evidence_id - 1;
                    let ev = test_case.evidence()[evidence_id].clone();

                    match ev.value().get_data(&mut package) {
                        Ok(data) => {
                            let mut cursor = Cursor::new(data);
                            io::copy(&mut cursor, &mut io::stdout())
                                .expect("failed to write to stdout");
                        }
                        Err(e) => return CliError::FailedToReadPackage(Rc::new(e)).into(),
                    }

                    // This is the ONE time we need to exit without going through the usual pathway.
                    std::process::exit(0);
                }
                Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
            }
        }
        TestCasesSubcommand::UpdateEvidence {
            case,
            evidence_id,
            evidence_value,
        } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let case_id = match_test_case(&mut package, case.clone());
                if case_id.is_none() {
                    return CliError::CannotMatchTestCase(case.clone()).into();
                }
                let case_id = case_id.unwrap();

                match evidence_from_evidence_value(evidence_value.clone(), &mut package) {
                    Ok(ev) => {
                        let test_case = package.test_case_mut(case_id).unwrap().unwrap();
                        if *evidence_id < 1 || *evidence_id > test_case.evidence().len() {
                            return CliError::CannotMatchEvidence(*evidence_id).into();
                        }
                        let evidence_id = *evidence_id - 1;
                        test_case.evidence_mut()[evidence_id] = ev;
                    }
                    Err(e) => return e.into(),
                }

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                CliData::TestCase(CliTestCase {
                    name: test_case.metadata().title().clone(),
                    executed_at: *test_case.metadata().execution_datetime(),
                    evidence: test_case
                        .evidence()
                        .iter()
                        .map(|ev| match ev.kind() {
                            EvidenceKind::Text => CliEvidence::Text {
                                data: String::from_utf8(ev.value().get_data(&mut package).unwrap())
                                    .unwrap(),
                            },
                            EvidenceKind::Image => CliEvidence::Image,
                            EvidenceKind::Http => CliEvidence::Http,
                            EvidenceKind::File => CliEvidence::File {
                                original_filename: ev.original_filename().clone(),
                            },
                        })
                        .collect(),
                })
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
        TestCasesSubcommand::DeleteEvidence { case, evidence_id } => {
            match EvidencePackage::open(path) {
                Ok(mut package) => {
                    let case_id = match_test_case(&mut package, case.clone());
                    if case_id.is_none() {
                        return CliError::CannotMatchTestCase(case.clone()).into();
                    }
                    let case_id = case_id.unwrap();

                    {
                        let test_case = package.test_case_mut(case_id).unwrap().unwrap();
                        if *evidence_id < 1 || *evidence_id > test_case.evidence().len() {
                            return CliError::CannotMatchEvidence(*evidence_id).into();
                        }
                        let evidence_id = *evidence_id - 1;
                        test_case.evidence_mut().remove(evidence_id);
                    }
                    if let Err(e) = package.save() {
                        return CliError::FailedToSavePackage(Rc::new(e)).into();
                    }

                    let test_case = package.test_case(case_id).unwrap().unwrap().clone();
                    CliData::TestCase(CliTestCase {
                        name: test_case.metadata().title().clone(),
                        executed_at: *test_case.metadata().execution_datetime(),
                        evidence: test_case
                            .evidence()
                            .iter()
                            .map(|ev| match ev.kind() {
                                EvidenceKind::Text => CliEvidence::Text {
                                    data: String::from_utf8(
                                        ev.value().get_data(&mut package).unwrap(),
                                    )
                                    .unwrap(),
                                },
                                EvidenceKind::Image => CliEvidence::Image,
                                EvidenceKind::Http => CliEvidence::Http,
                                EvidenceKind::File => CliEvidence::File {
                                    original_filename: ev.original_filename().clone(),
                                },
                            })
                            .collect(),
                    })
                }
                Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
            }
        }
    }
}

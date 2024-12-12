use std::{fmt, path::PathBuf, rc::Rc};

use chrono::FixedOffset;
use clap::Subcommand;
use colored::Colorize;
use evidenceangel::{Author, EvidencePackage};
use schemars::JsonSchema;
use serde::Serialize;

use crate::result::{CliData, CliError};

/// Subcommands to work on packages
#[derive(Subcommand, Clone)]
pub enum PackageSubcommand {
    /// Create a new package.
    Create {
        /// The title of the new package.
        #[arg(index = 1)]
        title: String,

        /// The description of the new package.
        #[arg(index = 2)]
        description: Option<String>,

        /// The authors of the new package, in 'Name <Email>' or just 'Name' format.
        #[arg(short, long)]
        authors: Vec<String>,
    },

    /// Read the data from a package.
    Read,

    /// Update the details of this package
    Update {
        /// The new title of the package.
        #[arg(short, long)]
        title: Option<String>,

        /// The new description of the package.
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add an author to this package
    AddAuthor {
        /// The new author for the package, in 'Name <Email>' or just 'Name' format.
        #[arg(index = 1)]
        author: String,
    },

    /// Delete an author from this package
    DeleteAuthor {
        /// The number (one-based index) of the author to remove
        #[arg(index = 1)]
        author: usize,
    },
}

/// A package, for display or JSON serialization
#[derive(Serialize, JsonSchema)]
pub struct CliPackage {
    /// The name of the package
    name: String,
    /// The description of the package
    description: Option<String>,
    /// The authors of the package
    authors: Vec<String>,
    /// The test cases within this package
    test_cases: Vec<PackageTestCase>,
}

impl CliPackage {
    /// Create a new package for display on screen or via JSON
    fn new(
        name: String,
        authors: Vec<String>,
        description: Option<String>,
        mut test_cases: Vec<PackageTestCase>,
    ) -> Self {
        test_cases.sort_by(|a, b| a.executed_at.cmp(&b.executed_at));
        CliPackage {
            name,
            authors,
            description,
            test_cases,
        }
    }
}

impl fmt::Display for CliPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📦 {}", self.name.bold())?;
        if let Some(description) = &self.description {
            writeln!(f, "  {}", description)?;
        }

        writeln!(f, "\nAuthors:")?;
        for (idx, author) in self.authors.iter().enumerate() {
            let ch = if idx == self.authors.len() - 1 {
                "╰"
            } else {
                "├"
            };
            writeln!(
                f,
                "  {} {} {}",
                ch,
                format!("[#{}]", idx + 1).blue(),
                author
            )?;
        }

        writeln!(f, "\nTest Cases:")?;
        for (idx, test_case) in self.test_cases.iter().enumerate() {
            let ch = if idx == self.test_cases.len() - 1 {
                "╰"
            } else {
                "├"
            };
            writeln!(
                f,
                "  {} {} {} {}",
                ch,
                format!("[#{}]", idx + 1).blue(),
                test_case.title,
                format!("({})", test_case.executed_at).magenta(),
            )?;
        }

        Ok(())
    }
}

/// A test case within a package
#[derive(Serialize, JsonSchema)]
struct PackageTestCase {
    /// The title of the test case
    title: String,
    /// The time the test case was executed
    executed_at: chrono::DateTime<FixedOffset>,
}

/// Process the package subcommand
pub fn process(path: PathBuf, command: &PackageSubcommand) -> CliData {
    match command {
        PackageSubcommand::Create {
            title,
            description,
            authors,
        } => {
            let mut manipulated_authors = vec![];
            for author in authors {
                if author.contains('<') && author.contains('>') {
                    let (name, email_and_finish_angle) = author.split_once('<').unwrap();
                    manipulated_authors.push(Author::new_with_email(
                        name.trim(),
                        email_and_finish_angle.trim_end_matches('>').trim(),
                    ));
                } else {
                    manipulated_authors.push(Author::new(author.trim()));
                }
            }

            match EvidencePackage::new_with_description(
                path,
                title.clone(),
                description.clone(),
                manipulated_authors,
            ) {
                Ok(package) => CliData::Package(CliPackage::new(
                    package.metadata().title().clone(),
                    package
                        .metadata()
                        .authors()
                        .iter()
                        .map(|a| a.to_string())
                        .collect(),
                    package.metadata().description().clone(),
                    package
                        .test_case_iter()
                        .unwrap()
                        .map(|tc| PackageTestCase {
                            title: tc.metadata().title().clone(),
                            executed_at: *tc.metadata().execution_datetime(),
                        })
                        .collect(),
                )),
                Err(e) => CliError::FailedToSavePackage(Rc::new(e)).into(),
            }
        }

        PackageSubcommand::Read => match EvidencePackage::open(path) {
            Ok(package) => CliData::Package(CliPackage::new(
                package.metadata().title().clone(),
                package
                    .metadata()
                    .authors()
                    .iter()
                    .map(|a| a.to_string())
                    .collect(),
                package.metadata().description().clone(),
                package
                    .test_case_iter()
                    .unwrap()
                    .map(|tc| PackageTestCase {
                        title: tc.metadata().title().clone(),
                        executed_at: *tc.metadata().execution_datetime(),
                    })
                    .collect(),
            )),
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },

        PackageSubcommand::Update { title, description } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                if let Some(title) = title {
                    package.metadata_mut().set_title(title.clone());
                }
                if let Some(description) = description {
                    package
                        .metadata_mut()
                        .set_description(Some(description.clone()));
                }

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                CliData::Package(CliPackage::new(
                    package.metadata().title().clone(),
                    package
                        .metadata()
                        .authors()
                        .iter()
                        .map(|a| a.to_string())
                        .collect(),
                    package.metadata().description().clone(),
                    package
                        .test_case_iter()
                        .unwrap()
                        .map(|tc| PackageTestCase {
                            title: tc.metadata().title().clone(),
                            executed_at: *tc.metadata().execution_datetime(),
                        })
                        .collect(),
                ))
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },

        PackageSubcommand::AddAuthor { author } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                let new_author = if author.contains('<') && author.contains('>') {
                    let (name, email_and_finish_angle) = author.split_once('<').unwrap();
                    Author::new_with_email(
                        name.trim(),
                        email_and_finish_angle.trim_end_matches('>').trim(),
                    )
                } else {
                    Author::new(author.trim())
                };
                package.metadata_mut().authors_mut().push(new_author);

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                CliData::Package(CliPackage::new(
                    package.metadata().title().clone(),
                    package
                        .metadata()
                        .authors()
                        .iter()
                        .map(|a| a.to_string())
                        .collect(),
                    package.metadata().description().clone(),
                    package
                        .test_case_iter()
                        .unwrap()
                        .map(|tc| PackageTestCase {
                            title: tc.metadata().title().clone(),
                            executed_at: *tc.metadata().execution_datetime(),
                        })
                        .collect(),
                ))
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },

        PackageSubcommand::DeleteAuthor { author } => match EvidencePackage::open(path) {
            Ok(mut package) => {
                package.metadata_mut().authors_mut().remove(*author - 1);

                if let Err(e) = package.save() {
                    return CliError::FailedToSavePackage(Rc::new(e)).into();
                }

                CliData::Package(CliPackage::new(
                    package.metadata().title().clone(),
                    package
                        .metadata()
                        .authors()
                        .iter()
                        .map(|a| a.to_string())
                        .collect(),
                    package.metadata().description().clone(),
                    package
                        .test_case_iter()
                        .unwrap()
                        .map(|tc| PackageTestCase {
                            title: tc.metadata().title().clone(),
                            executed_at: *tc.metadata().execution_datetime(),
                        })
                        .collect(),
                ))
            }
            Err(e) => CliError::FailedToReadPackage(Rc::new(e)).into(),
        },
    }
}

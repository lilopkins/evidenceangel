use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use evidenceangel::{Author, Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile};
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to work with
    #[arg(index = 1)]
    file: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Create a new package.
    Create {
        /// The title of the new package.
        #[arg(index = 1)]
        title: String,

        /// The authors of the new package, in 'Name <Email>' or just 'Name' format.
        #[arg(short, long)]
        authors: Vec<String>,
    },
    /// Read the data from a package.
    Read,
    /// Create a new test case.
    CreateTestCase {
        /// The title of the new test case.
        #[arg(index = 1)]
        title: String,
    },
    /// View a test case.
    ViewTestCase {
        /// The ID of the test case to view.
        #[arg(index = 1)]
        case_id: Uuid,
    },
    /// Delete a test case from a package.
    DeleteTestCase {
        /// The ID of the test case to delete.
        #[arg(index = 1)]
        case_id: Uuid,
    },
    /// Add evidence to a test case.
    AddEvidence {
        /// The ID of the test case to add to.
        #[arg(index = 1)]
        case_id: Uuid,
        /// The evidence to add
        #[command(subcommand)]
        evidence_value: EvidenceValue,
    },
    /// Delete evidence from a test case.
    DeleteEvidence {
        /// The ID of the test case to add to.
        #[arg(index = 1)]
        case_id: Uuid,
        /// The index of the evidence to delete.
        #[arg(index = 2)]
        evidence_id: usize,
    },
}

#[derive(Subcommand, Clone)]
enum EvidenceValue {
    /// Text-based evidence
    Text {
        /// The text to add
        #[arg(index = 1)]
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
}

fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    pretty_env_logger::init();

    match args.command {
        Command::Create { title, authors } => {
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

            let package = EvidencePackage::new(args.file, title, manipulated_authors)?;
            println!("{package:#?}");
            eprintln!("Package created.");
        }
        Command::Read => {
            let package = EvidencePackage::open(args.file)?;
            println!("{package:#?}");
        }
        Command::CreateTestCase { title } => {
            let mut package = EvidencePackage::open(args.file)?;
            let new_case = package.create_test_case(title)?;
            let new_id = *new_case.id();
            if let Err(e) = package.save() {
                eprintln!("Failed to create test case: {e}");
            } else {
                println!("{}", new_id);
            }
        }
        Command::ViewTestCase { case_id } => {
            let mut package = EvidencePackage::open(args.file)?;
            if let Some(case) = package.test_case(case_id)? {
                println!("{case:#?}");
            }
        }
        Command::DeleteTestCase { case_id } => {
            let mut package = EvidencePackage::open(args.file)?;
            if !package.delete_test_case(case_id)? {
                eprintln!("Nothing deleted!");
            }
            if let Err(e) = package.save() {
                eprintln!("Failed to delete test case: {e}");
            }
        }
        Command::AddEvidence {
            case_id,
            evidence_value,
        } => {
            let mut package = EvidencePackage::open(args.file)?;
            match evidence_value {
                EvidenceValue::Text { value } => {
                    if let Some(case) = package.test_case_mut(case_id)? {
                        case.evidence_mut().push(Evidence::new(
                            EvidenceKind::Text,
                            EvidenceData::Text { content: value },
                        ));
                    } else {
                        eprintln!("No test case exists with that ID!");
                    }
                }
                EvidenceValue::Image { image, caption } => {
                    let media: MediaFile = fs::read(image)?.into();
                    let hash = media.hash();
                    if let Some(mime) = media.mime_type() {
                        if !mime.mime_type().starts_with("image/") {
                            eprintln!("The provided file was not an image!");
                            return Ok(());
                        }
                    } else {
                        eprintln!("The provided file was not an image!");
                        return Ok(());
                    }
                    package.add_media(media)?;
                    if let Some(case) = package.test_case_mut(case_id)? {
                        let mut evidence =
                            Evidence::new(EvidenceKind::Image, EvidenceData::Media { hash });
                        evidence.set_caption(caption);
                        case.evidence_mut().push(evidence);
                    } else {
                        eprintln!("No test case exists with that ID!");
                    }
                }
            }
            if let Err(e) = package.save() {
                eprintln!("Failed to add evidence: {e}");
            }
        }
        Command::DeleteEvidence {
            case_id,
            evidence_id,
        } => {
            let mut package = EvidencePackage::open(args.file)?;
            if let Some(case) = package.test_case_mut(case_id)? {
                if evidence_id > case.evidence().len() {
                    eprintln!("No evidence exists with that index!");
                } else {
                    case.evidence_mut().remove(evidence_id);
                }

                if let Err(e) = package.save() {
                    eprintln!("Failed to delete evidence: {e}");
                }
            } else {
                eprintln!("No test case exists with that ID!");
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}");
    }
}

use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::{CommandFactory, Parser, Subcommand};
use evidenceangel::{
    exporters::{excel::ExcelExporter, html::HtmlExporter, Exporter},
    Author, Evidence, EvidenceData, EvidenceKind, EvidencePackage, MediaFile,
};
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
    /// Print shell completions. This will not read the file specified, so it can be
    /// an invalid name.
    ShellCompletions {
        /// The shell to generate for.
        #[arg(index = 1)]
        shell: clap_complete::Shell,
    },
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
    /// Export to another format.
    ExportPackage {
        /// The format to export to. Permitted values are "html" and "excel".
        #[arg(index = 1)]
        format: String,
        /// The target file to write.
        #[arg(index = 2)]
        target: PathBuf,
    },
    /// Export a test case to another format.
    ExportTestCase {
        /// The ID of the test case to export.
        #[arg(index = 1)]
        case_id: Uuid,
        /// The format to export to. Permitted values are "html" and "excel".
        #[arg(index = 2)]
        format: String,
        /// The target file to write.
        #[arg(index = 3)]
        target: PathBuf,
    },
}

#[derive(Subcommand, Clone)]
enum EvidenceValue {
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
        /// The text to add, or `-` to read from stdin.
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

fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    pretty_env_logger::init();

    match args.command {
        Command::ShellCompletions { shell } => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
        }
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
            let package = EvidencePackage::open(args.file)?;
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
                EvidenceValue::Text { mut value } => {
                    let mut buf = vec![];
                    if value == "-" {
                        io::stdin().read_to_end(&mut buf)?;
                        value = String::from_utf8_lossy(&buf).into_owned();
                    }
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
                        if !["image/png", "image/jpeg"].contains(&mime.mime_type()) {
                            eprintln!("The provided file was not a valid PNG or JPEG!");
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
                EvidenceValue::Http { value, caption } => {
                    let mut buf = vec![];
                    if value == "-" {
                        io::stdin().read_to_end(&mut buf)?;
                    } else {
                        buf = value.into_bytes();
                    }
                    if let Some(case) = package.test_case_mut(case_id)? {
                        let mut evidence =
                            Evidence::new(EvidenceKind::Http, EvidenceData::Base64 { data: buf });
                        evidence.set_caption(caption);
                        case.evidence_mut().push(evidence);
                    } else {
                        eprintln!("No test case exists with that ID!");
                    }
                }
                EvidenceValue::File { path, caption } => {
                    let media: MediaFile = fs::read(&path)?.into();
                    let hash = media.hash();
                    package.add_media(media)?;
                    if let Some(case) = package.test_case_mut(case_id)? {
                        let mut evidence =
                            Evidence::new(EvidenceKind::File, EvidenceData::Media { hash });
                        evidence.set_caption(caption);
                        evidence.set_original_filename(
                            path.file_name().map(|s| s.to_string_lossy().to_string()),
                        );
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
        Command::ExportPackage { format, target } => {
            let mut package = EvidencePackage::open(args.file)?;
            match format.as_str() {
                "html" => {
                    HtmlExporter.export_package(&mut package, target)?;
                }
                "excel" => {
                    ExcelExporter.export_package(&mut package, target)?;
                }
                _ => eprintln!("Invalid format specified."),
            }
        }
        Command::ExportTestCase {
            case_id,
            format,
            target,
        } => {
            let mut package = EvidencePackage::open(args.file)?;
            match format.as_str() {
                "html" => {
                    HtmlExporter.export_case(&mut package, case_id, target)?;
                }
                "excel" => {
                    ExcelExporter.export_case(&mut package, case_id, target)?;
                }
                _ => eprintln!("Invalid format specified."),
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

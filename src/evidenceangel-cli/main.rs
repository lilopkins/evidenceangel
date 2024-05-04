use std::path::PathBuf;

use clap::{Parser, Subcommand};
use evidenceangel::{Author, EvidencePackage};

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
}

fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::Create { title, authors } => {
            let mut manipulated_authors = vec![];
            for author in authors {
                if author.contains("<") && author.contains(">") {
                    let (name, email_and_finish_angle) = author.split_once("<").unwrap();
                    manipulated_authors.push(Author::new_with_email(
                        name.trim(),
                        email_and_finish_angle.trim_end_matches(">").trim(),
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
    }

    Ok(())
}

fn main() {
    if let Err(e) = exec() {
        eprintln!("{e}");
    }
}

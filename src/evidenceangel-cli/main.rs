#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ref_option)]

//! # EvidenceAngel CLI
//!
//! ## Usage
//!
//! To get usage information, please execute the binary with `--help`.

use std::io;

use arg_parser::{Args, Command};
use clap::{CommandFactory, Parser};
use result::{CliData, CliError, CliOutput};

/// Module containing utility functions for working with angelmark.
mod angelmark;
/// Module containing the argument parser for this CLI tool.
mod arg_parser;

/// Module containing functionality for working with exporting.
mod export;
/// Module containing functionality for working with packages.
mod package;
/// Module containing serializable and presentable result data.
mod result;
/// Module containing functionality for working with test cases.
mod test_cases;

fn main() {
    let args = Args::parse();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_ansi(true)
        .with_max_level(*args.verbose())
        .with_writer(std::sync::Mutex::new(io::stderr()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("couldn't start logging");

    if *args.no_color() {
        colored::control::set_override(false);
    }

    // Validate CLI argument combinations
    if let Some(r) = validate_cli_combinations(&args) {
        r.output(&args);
        return;
    }

    // Handle the subcommands first that aren't compatible with JSON output
    match args.command() {
        Command::ShellCompletions {
            shell,
            command_name,
        } => {
            let mut cmd = <arg_parser::Args as CommandFactory>::command();
            let name = command_name.clone().unwrap_or(cmd.get_name().to_string());
            clap_complete::generate(*shell, &mut cmd, name, &mut io::stdout());
            std::process::exit(0);
        }
        Command::JsonSchema => {
            println!("{}", CliData::schema());
            std::process::exit(0);
        }
        _ => (),
    }

    // Now handle the rest...
    let path = args.file().clone().unwrap();
    let result: CliData = match args.command() {
        Command::ShellCompletions { .. } | Command::JsonSchema => unreachable!(),
        Command::Package { command } => package::process(path, command),
        Command::TestCases { command } => test_cases::process(path, command),
        Command::Export { command } => export::process(path, command),
    };

    result.output(&args);
}

/// Before triggering any activity, check that the provided combinations of command line arguments are valid.
fn validate_cli_combinations(args: &Args) -> Option<CliData> {
    match args.command() {
        Command::ShellCompletions { .. } | Command::JsonSchema => {
            if *args.json() {
                return Some(CliError::CannotBeSerialized.into());
            }
        }
        _ => {
            if args.file().is_none() {
                return Some(CliError::MissingFile.into());
            }
        }
    }
    None
}

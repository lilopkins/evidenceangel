use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use getset::Getters;

use crate::{
    export::ExportSubcommand, package::PackageSubcommand, test_cases::TestCasesSubcommand,
};

/// The command line arguments for this tool
#[derive(Parser, Getters)]
#[command(version, about, long_about = None)]
#[getset(get = "pub")]
pub struct Args {
    /// Output as JSON data
    #[arg(short, long)]
    json: bool,

    /// Disable color in outputs
    #[arg(long)]
    no_color: bool,

    /// The file to work with
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// The action for this tool to perform
    #[command(subcommand)]
    command: Command,

    /// Verbosity
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Print shell completions.
    ShellCompletions {
        /// The shell to generate for.
        #[arg(index = 1)]
        shell: clap_complete::Shell,

        /// The name of the CLI executable, if different from as built.
        #[arg(index = 2)]
        command_name: Option<String>,
    },
    /// Print the JSON schema of the serialized output
    JsonSchema,
    /// Work with packages
    Package {
        /// The operation to perform on a package
        #[command(subcommand)]
        command: PackageSubcommand,
    },
    /// Work with test cases
    TestCases {
        /// The operation to perform on test cases in a package
        #[command(subcommand)]
        command: TestCasesSubcommand,
    },
    /// Export packages and test cases
    Export {
        /// The operation to perform on test cases in a package
        #[command(subcommand)]
        command: ExportSubcommand,
    },
}

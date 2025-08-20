/// CLI tool data
mod data;
/// CLI tool errors
mod error;
/// Traits to allow easy output
mod traits;

pub use data::CliData;
pub use error::CliError;
pub use traits::CliOutput;

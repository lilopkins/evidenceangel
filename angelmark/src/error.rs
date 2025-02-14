use thiserror::Error;

/// AngelMark errors
#[derive(Error, Debug)]
pub enum Error {
    /// A parsing error
    #[error("Error parsing the provided markup")]
    Parsing(#[from] Box<pest::error::Error<crate::lexer::Rule>>),
}

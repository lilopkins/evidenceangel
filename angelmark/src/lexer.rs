use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "angelmark.pest"]
pub struct AngelmarkParser;

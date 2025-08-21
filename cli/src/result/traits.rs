use crate::arg_parser::Args;

use super::data::CliData;

/// Structs inheriting this trait are define functions to output their data in
/// both a pretty terminal environment and in a serializable JSON format.
pub trait CliOutput {
    /// Generate the output for printing to stdout
    fn output_pretty(&self) -> String;
    /// Generate JSON output for printing to stdout
    fn output_json(&self) -> String;
    /// Output the result
    fn output(&self, args: &Args) {
        if *args.json() {
            println!("{}", self.output_json());
        } else {
            print!("{}", self.output_pretty());
        }
    }
}

impl CliOutput for CliData {
    fn output_pretty(&self) -> String {
        self.to_string()
    }

    fn output_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

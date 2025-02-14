use angelmark::AngelmarkText;
use colored::Colorize;

/// Convert [`AngelmarkText`] to a string with ANSI symbols for terminal display.
pub(crate) fn angelmark_to_term(angelmark: &AngelmarkText) -> String {
    match angelmark {
        AngelmarkText::Raw(txt) => txt.clone(),
        AngelmarkText::Bold(content) => angelmark_to_term(content).bold().to_string(),
        AngelmarkText::Italic(content) => angelmark_to_term(content).italic().to_string(),
        AngelmarkText::Monospace(content) => format!("`{}`", angelmark_to_term(content)),
    }
}

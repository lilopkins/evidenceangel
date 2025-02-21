use angelmark::AngelmarkText;
use colored::Colorize;

/// Convert [`AngelmarkText`] to a string with ANSI symbols for terminal display.
pub(crate) fn angelmark_to_term(angelmark: &AngelmarkText) -> String {
    match angelmark {
        AngelmarkText::Raw(txt, _span) => txt.clone(),
        AngelmarkText::Bold(content, _span) => angelmark_to_term(content).bold().to_string(),
        AngelmarkText::Italic(content, _span) => angelmark_to_term(content).italic().to_string(),
        AngelmarkText::Monospace(content, _span) => format!("`{}`", angelmark_to_term(content)),
    }
}

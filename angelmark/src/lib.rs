#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod error;

pub use error::Error;
use getset::Getters;
use pest::{iterators::Pair, Parser, Span};

mod lexer;
use lexer::Rule;
use regex::Regex;

/// A parsed span with an owned clone of it's matched text
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct OwnedSpan {
    /// The original matched text
    original: String,
    /// The matched span
    span: (usize, usize),
}

impl From<Span<'_>> for OwnedSpan {
    fn from(value: Span<'_>) -> Self {
        Self {
            original: value.as_str().to_owned(),
            span: (value.start(), value.end()),
        }
    }
}

/// A line of markup in AngelMark
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkLine {
    /// A level 1 heading.
    Heading1(Vec<AngelmarkText>, OwnedSpan),
    /// A level 2 heading.
    Heading2(Vec<AngelmarkText>, OwnedSpan),
    /// A level 3 heading.
    Heading3(Vec<AngelmarkText>, OwnedSpan),
    /// A level 4 heading.
    Heading4(Vec<AngelmarkText>, OwnedSpan),
    /// A level 5 heading.
    Heading5(Vec<AngelmarkText>, OwnedSpan),
    /// A level 6 heading.
    Heading6(Vec<AngelmarkText>, OwnedSpan),
    /// A line of text.
    TextLine(AngelmarkText, OwnedSpan),
    /// A line separator.
    Newline(OwnedSpan),
}

impl AngelmarkLine {
    /// Get the span from this line
    #[must_use]
    pub fn span(&self) -> &OwnedSpan {
        match self {
            Self::Heading1(_, span)
            | Self::Heading2(_, span)
            | Self::Heading3(_, span)
            | Self::Heading4(_, span)
            | Self::Heading5(_, span)
            | Self::Heading6(_, span)
            | Self::TextLine(_, span)
            | Self::Newline(span) => span,
        }
    }

    /// Compare two [`AngelmarkLine`] instances, ignoring their span.
    #[must_use]
    pub fn eq_ignoring_span(&self, other: &Self) -> bool {
        match self {
            Self::Heading1(inner, _) => {
                if let Self::Heading1(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading2(inner, _) => {
                if let Self::Heading2(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading3(inner, _) => {
                if let Self::Heading3(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading4(inner, _) => {
                if let Self::Heading4(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading5(inner, _) => {
                if let Self::Heading5(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading6(inner, _) => {
                if let Self::Heading6(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::TextLine(inner, _) => {
                if let Self::TextLine(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Newline(_) => {
                matches!(other, Self::Newline(_))
            }
        }
    }
}

/// Textual content
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkText {
    /// Raw text
    Raw(String, OwnedSpan),
    /// Bold content
    Bold(Box<AngelmarkText>, OwnedSpan),
    /// Italicised content
    Italic(Box<AngelmarkText>, OwnedSpan),
    /// Monospace content
    Monospace(Box<AngelmarkText>, OwnedSpan),
}

impl AngelmarkText {
    /// Get the span from this text
    #[must_use]
    pub fn span(&self) -> &OwnedSpan {
        match self {
            Self::Raw(_, span)
            | Self::Bold(_, span)
            | Self::Italic(_, span)
            | Self::Monospace(_, span) => span,
        }
    }

    /// Compare two [`AngelmarkText`] instances, ignoring their span.
    #[must_use]
    pub fn eq_ignoring_span(&self, other: &Self) -> bool {
        match self {
            Self::Raw(inner, _) => {
                if let Self::Raw(other_inner, _) = other {
                    inner == other_inner
                } else {
                    false
                }
            }
            Self::Bold(inner, _) => {
                if let Self::Bold(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Italic(inner, _) => {
                if let Self::Italic(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Monospace(inner, _) => {
                if let Self::Monospace(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
        }
    }
}

/// Parse an Angelmark markup string into a programmatically sensible interface.
///
/// # Errors
///
/// - [`Error::Parsing`] if the input markup couldn't be parsed.
#[allow(clippy::missing_panics_doc)]
pub fn parse_angelmark<S: AsRef<str>>(input: S) -> Result<Vec<AngelmarkLine>, Error> {
    let markup_file = lexer::AngelmarkParser::parse(Rule::MarkupFile, input.as_ref())
        .map_err(Box::new)?
        .next()
        .unwrap();

    let mut content = vec![];

    for pair in markup_file.into_inner() {
        let span = pair.as_span();
        match pair.as_rule() {
            Rule::EOI => break,
            Rule::Comment => (),
            Rule::Newline => {
                if let Some(AngelmarkLine::Newline(_)) = content.last() {
                } else {
                    content.push(AngelmarkLine::Newline(pair.as_span().into()));
                }
            }

            Rule::Heading1 => content.push(AngelmarkLine::Heading1(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::Heading2 => content.push(AngelmarkLine::Heading2(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::Heading3 => content.push(AngelmarkLine::Heading3(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::Heading4 => content.push(AngelmarkLine::Heading4(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::Heading5 => content.push(AngelmarkLine::Heading5(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::Heading6 => content.push(AngelmarkLine::Heading6(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
                span.into(),
            )),
            Rule::TextBold | Rule::TextItalic | Rule::TextMonospace | Rule::RawText => {
                content.push(AngelmarkLine::TextLine(
                    parse_text_content(pair),
                    span.into(),
                ));
            }

            _ => unreachable!(),
        }
    }

    Ok(content)
}

fn parse_text_content(pair: Pair<Rule>) -> AngelmarkText {
    assert!([
        Rule::TextBold,
        Rule::TextItalic,
        Rule::TextMonospace,
        Rule::RawText,
    ]
    .contains(&pair.as_rule()));

    let span = pair.as_span();
    match pair.as_rule() {
        Rule::TextBold => AngelmarkText::Bold(
            Box::new(parse_text_content(pair.into_inner().next().unwrap())),
            span.into(),
        ),
        Rule::TextItalic => AngelmarkText::Italic(
            Box::new(parse_text_content(pair.into_inner().next().unwrap())),
            span.into(),
        ),
        Rule::TextMonospace => AngelmarkText::Monospace(
            Box::new(parse_text_content(pair.into_inner().next().unwrap())),
            span.into(),
        ),
        Rule::RawText => AngelmarkText::Raw(unescape_str(pair.as_str()), span.into()),

        _ => unreachable!(),
    }
}

fn unescape_str(s: &str) -> String {
    let r = Regex::new(r"\\(.)").unwrap();
    r.replace_all(s, "$1").into_owned()
}

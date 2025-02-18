#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod error;
pub use error::Error;
use pest::{iterators::Pair, Parser};

mod lexer;
use lexer::Rule;

/// A line of markup in AngelMark
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkLine {
    /// A level 1 heading.
    Heading1(Vec<AngelmarkText>),
    /// A level 2 heading.
    Heading2(Vec<AngelmarkText>),
    /// A level 3 heading.
    Heading3(Vec<AngelmarkText>),
    /// A level 4 heading.
    Heading4(Vec<AngelmarkText>),
    /// A level 5 heading.
    Heading5(Vec<AngelmarkText>),
    /// A level 6 heading.
    Heading6(Vec<AngelmarkText>),
    /// A line of text.
    TextLine(AngelmarkText),
    /// A line separator.
    Newline,
}

/// Textual content
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkText {
    /// Raw text
    Raw(String),
    /// Bold content
    Bold(Box<AngelmarkText>),
    /// Italicised content
    Italic(Box<AngelmarkText>),
    /// Monospace content
    Monospace(Box<AngelmarkText>),
}

/// Parse an Angelmark markup string into a programmatically sensible interface.
///
/// # Errors
///
/// - [`Error::Parsing`] if the input markup couldn't be parsed.
pub fn parse_angelmark<S: AsRef<str>>(input: S) -> Result<Vec<AngelmarkLine>, Error> {
    let markup_file = lexer::AngelmarkParser::parse(Rule::MarkupFile, input.as_ref())
        .map_err(Box::new)?
        .next()
        .unwrap();

    let mut content = vec![];

    for pair in markup_file.into_inner() {
        let _rule = pair.as_rule();
        match pair.as_rule() {
            Rule::EOI => break,
            Rule::Comment => (),
            Rule::Newline => {
                if content.last() != Some(&AngelmarkLine::Newline) {
                    content.push(AngelmarkLine::Newline)
                }
            }

            Rule::Heading1 => content.push(AngelmarkLine::Heading1(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::Heading2 => content.push(AngelmarkLine::Heading2(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::Heading3 => content.push(AngelmarkLine::Heading3(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::Heading4 => content.push(AngelmarkLine::Heading4(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::Heading5 => content.push(AngelmarkLine::Heading5(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::Heading6 => content.push(AngelmarkLine::Heading6(
                pair.into_inner()
                    .filter(|pair| pair.as_rule() != Rule::Newline)
                    .map(|pair| parse_text_content(pair))
                    .collect(),
            )),
            Rule::TextBold | Rule::TextItalic | Rule::TextMonospace | Rule::RawText => {
                content.push(AngelmarkLine::TextLine(parse_text_content(pair)))
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

    match pair.as_rule() {
        Rule::TextBold => AngelmarkText::Bold(Box::new(parse_text_content(
            pair.into_inner().next().unwrap(),
        ))),
        Rule::TextItalic => AngelmarkText::Italic(Box::new(parse_text_content(
            pair.into_inner().next().unwrap(),
        ))),
        Rule::TextMonospace => AngelmarkText::Monospace(Box::new(parse_text_content(
            pair.into_inner().next().unwrap(),
        ))),
        Rule::RawText => AngelmarkText::Raw(pair.as_str().to_string()),

        _ => unreachable!(),
    }
}

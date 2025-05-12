#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod error;
pub use error::Error;

mod lexer;
use lexer::Rule;

mod line;
pub use line::AngelmarkLine;

mod table;
pub use table::{
    AngelmarkTable, AngelmarkTableAlignment, AngelmarkTableAlignmentCell,
    AngelmarkTableAlignmentRow, AngelmarkTableCell, AngelmarkTableRow,
};

mod text;
pub use text::AngelmarkText;

mod traits;
pub use traits::EqIgnoringSpan;

use getset::Getters;
use pest::{iterators::Pair, Parser, Span};
use regex::Regex;

/// A parsed span with an owned clone of it's matched text
#[derive(Clone, Default, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct OwnedSpan {
    /// The original matched text
    original: String,
    /// The matched span
    span: (usize, usize),
}

impl std::fmt::Debug for OwnedSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedSpan").finish_non_exhaustive()
    }
}

impl From<Span<'_>> for OwnedSpan {
    fn from(value: Span<'_>) -> Self {
        Self {
            original: value.as_str().to_owned(),
            span: (value.start(), value.end()),
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
            Rule::Table => content.push(AngelmarkLine::Table(
                AngelmarkTable::from(pair),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_valid() {
        parse_angelmark("").unwrap();
    }

    #[test]
    fn single_line_with_no_newline_is_valid() {
        parse_angelmark("test").unwrap();
    }

    #[test]
    fn parse_test_document() {
        let parsed_doc = parse_angelmark(include_str!("../tests/angelmark.md")).unwrap();

        let expected = vec![
            AngelmarkLine::Heading1(
                vec![AngelmarkText::Raw(
                    "Heading 1".to_string(),
                    OwnedSpan::default(),
                )],
                OwnedSpan::default(),
            ),
            AngelmarkLine::Heading2(
                vec![
                    AngelmarkText::Bold(
                        Box::new(AngelmarkText::Raw(
                            "Heading".to_string(),
                            OwnedSpan::default(),
                        )),
                        OwnedSpan::default(),
                    ),
                    AngelmarkText::Raw(" 2".to_string(), OwnedSpan::default()),
                ],
                OwnedSpan::default(),
            ),
            AngelmarkLine::Heading3(
                vec![
                    AngelmarkText::Italic(
                        Box::new(AngelmarkText::Raw(
                            "Heading".to_string(),
                            OwnedSpan::default(),
                        )),
                        OwnedSpan::default(),
                    ),
                    AngelmarkText::Raw(" 3".to_string(), OwnedSpan::default()),
                ],
                OwnedSpan::default(),
            ),
            AngelmarkLine::Heading4(
                vec![AngelmarkText::Raw(
                    "Heading 4".to_string(),
                    OwnedSpan::default(),
                )],
                OwnedSpan::default(),
            ),
            AngelmarkLine::Heading5(
                vec![AngelmarkText::Raw(
                    "Heading 5".to_string(),
                    OwnedSpan::default(),
                )],
                OwnedSpan::default(),
            ),
            AngelmarkLine::Heading6(
                vec![AngelmarkText::Raw(
                    "Heading 6".to_string(),
                    OwnedSpan::default(),
                )],
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Bold(
                    Box::new(AngelmarkText::Raw("Bold".to_string(), OwnedSpan::default())),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Italic(
                    Box::new(AngelmarkText::Raw(
                        "Italic".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Bold(
                    Box::new(AngelmarkText::Italic(
                        Box::new(AngelmarkText::Raw(
                            "Bold and italic".to_string(),
                            OwnedSpan::default(),
                        )),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Italic(
                    Box::new(AngelmarkText::Raw(
                        "also italic".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Bold(
                    Box::new(AngelmarkText::Italic(
                        Box::new(AngelmarkText::Raw(
                            "bold and italic".to_string(),
                            OwnedSpan::default(),
                        )),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Italic(
                    Box::new(AngelmarkText::Bold(
                        Box::new(AngelmarkText::Raw(
                            "bold and italic".to_string(),
                            OwnedSpan::default(),
                        )),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw("Formatting ".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Bold(
                    Box::new(AngelmarkText::Raw("in".to_string(), OwnedSpan::default())),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw(" a line ".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Italic(
                    Box::new(AngelmarkText::Raw(
                        "as well".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw(" as ".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Italic(
                    Box::new(AngelmarkText::Raw(
                        "on it's own".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw("!".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Monospace(
                    Box::new(AngelmarkText::Raw(
                        "monospace".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw(
                    r#"Something with_underlines_separating_it but that\ shouldn't be italicised!"#
                        .to_string(),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::Table(
                AngelmarkTable {
                    rows: vec![
                        AngelmarkTableRow {
                            cells: vec![
                                AngelmarkTableCell {
                                    content: vec![
                                        AngelmarkText::Bold(
                                            Box::new(AngelmarkText::Raw(
                                                "Test Case".to_string(),
                                                OwnedSpan::default(),
                                            )),
                                            OwnedSpan::default(),
                                        ),
                                        AngelmarkText::Raw(" ".to_string(), OwnedSpan::default()),
                                    ],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Raw(
                                        " Objective ".to_string(),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Raw(
                                        " Expected Result".to_string(),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                            ],
                            span: OwnedSpan::default(),
                        },
                        AngelmarkTableRow {
                            cells: vec![
                                AngelmarkTableCell {
                                    content: vec![
                                        AngelmarkText::Raw("TC".to_string(), OwnedSpan::default()),
                                        AngelmarkText::Italic(
                                            Box::new(AngelmarkText::Raw(
                                                "01".to_string(),
                                                OwnedSpan::default(),
                                            )),
                                            OwnedSpan::default(),
                                        ),
                                    ],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Raw(
                                        "DEF".to_string(),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                            ],
                            span: OwnedSpan::default(),
                        },
                        AngelmarkTableRow {
                            cells: vec![
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Italic(
                                        Box::new(AngelmarkText::Raw(
                                            "TC02".to_string(),
                                            OwnedSpan::default(),
                                        )),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Raw(
                                        "HIJ".to_string(),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                                AngelmarkTableCell {
                                    content: vec![AngelmarkText::Raw(
                                        "KLM".to_string(),
                                        OwnedSpan::default(),
                                    )],
                                    span: OwnedSpan::default(),
                                },
                            ],
                            span: OwnedSpan::default(),
                        },
                    ],
                    width: 3,
                    alignment: AngelmarkTableAlignmentRow {
                        column_alignments: vec![
                            AngelmarkTableAlignmentCell {
                                alignment: AngelmarkTableAlignment::Right,
                                span: OwnedSpan::default(),
                            },
                            AngelmarkTableAlignmentCell {
                                alignment: AngelmarkTableAlignment::Left,
                                span: OwnedSpan::default(),
                            },
                            AngelmarkTableAlignmentCell {
                                alignment: AngelmarkTableAlignment::Center,
                                span: OwnedSpan::default(),
                            },
                        ],
                        span: OwnedSpan::default(),
                    },
                    span: OwnedSpan::default(),
                },
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw("Also ".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Monospace(
                    Box::new(AngelmarkText::Raw(
                        "monospace".to_string(),
                        OwnedSpan::default(),
                    )),
                    OwnedSpan::default(),
                ),
                OwnedSpan::default(),
            ),
            AngelmarkLine::TextLine(
                AngelmarkText::Raw(" but in a line.".to_string(), OwnedSpan::default()),
                OwnedSpan::default(),
            ),
            AngelmarkLine::Newline(OwnedSpan::default()),
        ];

        eprintln!("Parsed:");
        eprintln!("{parsed_doc:#?}");

        eprintln!("\nExpected (ignoring spans):");
        eprintln!("{expected:#?}");

        // Compare ignoring spans
        let mut parsed_iter = parsed_doc.iter();
        for item in expected {
            let parsed_item = parsed_iter.next().expect("not enough items were parsed");
            assert!(item.eq_ignoring_span(parsed_item));
        }
    }
}

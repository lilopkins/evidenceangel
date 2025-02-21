use angelmark::{parse_angelmark, AngelmarkLine, AngelmarkText, OwnedSpan};

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
    let parsed_doc = parse_angelmark(include_str!("angelmark.md")).unwrap();

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

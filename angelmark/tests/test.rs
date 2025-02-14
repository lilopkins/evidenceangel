use angelmark::{parse_angelmark, AngelmarkLine, AngelmarkText};

#[test]
fn empty_string_is_valid() {
    parse_angelmark("").unwrap();
}

#[test]
fn parse_test_document() {
    let parsed_doc = parse_angelmark(include_str!("angelmark.md")).unwrap();

    assert_eq!(
        parsed_doc,
        vec![
            AngelmarkLine::Heading1(AngelmarkText::Raw("Heading 1".to_string())),
            AngelmarkLine::Heading2(AngelmarkText::Raw("Heading 2".to_string())),
            AngelmarkLine::Heading3(AngelmarkText::Raw("Heading 3".to_string())),
            AngelmarkLine::Heading4(AngelmarkText::Raw("Heading 4".to_string())),
            AngelmarkLine::Heading5(AngelmarkText::Raw("Heading 5".to_string())),
            AngelmarkLine::Heading6(AngelmarkText::Raw("Heading 6".to_string())),
            AngelmarkLine::TextLine(AngelmarkText::Bold(Box::new(AngelmarkText::Raw(
                "Bold".to_string()
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Italic(Box::new(AngelmarkText::Raw(
                "Italic".to_string()
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Bold(Box::new(AngelmarkText::Italic(
                Box::new(AngelmarkText::Raw("Bold and italic".to_string()))
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Italic(Box::new(AngelmarkText::Raw(
                "also italic".to_string()
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Bold(Box::new(AngelmarkText::Italic(
                Box::new(AngelmarkText::Raw("bold and italic".to_string()))
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Italic(Box::new(AngelmarkText::Bold(
                Box::new(AngelmarkText::Raw("bold and italic".to_string()))
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Raw("Formatting ".to_string())),
            AngelmarkLine::TextLine(AngelmarkText::Bold(Box::new(AngelmarkText::Raw(
                "in".to_string()
            )))),
            AngelmarkLine::TextLine(AngelmarkText::Raw(" a line ".to_string())),
            AngelmarkLine::TextLine(AngelmarkText::Italic(Box::new(AngelmarkText::Raw(
                "as well".to_string()
            )))),
            AngelmarkLine::TextLine(AngelmarkText::Raw(" as ".to_string())),
            AngelmarkLine::TextLine(AngelmarkText::Italic(Box::new(AngelmarkText::Raw(
                "on it's own".to_string()
            )))),
            AngelmarkLine::TextLine(AngelmarkText::Raw("!".to_string())),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Monospace(Box::new(AngelmarkText::Raw(
                "monospace".to_string()
            )))),
            AngelmarkLine::Newline,
            AngelmarkLine::TextLine(AngelmarkText::Raw("Also ".to_string())),
            AngelmarkLine::TextLine(AngelmarkText::Monospace(Box::new(AngelmarkText::Raw(
                "monospace".to_string()
            )))),
            AngelmarkLine::TextLine(AngelmarkText::Raw(" but in a line.".to_string())),
            AngelmarkLine::Newline,
        ]
    );
}

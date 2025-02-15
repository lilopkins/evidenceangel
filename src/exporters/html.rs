use std::fs;

use angelmark::{parse_angelmark, AngelmarkLine, AngelmarkText};
use base64::Engine;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag};
use uuid::Uuid;

use crate::{EvidenceData, EvidenceKind, EvidencePackage, MediaFile, TestCase};

use super::Exporter;

/// An exporter to HTML document.
#[derive(Default)]
pub struct HtmlExporter;

impl Exporter for HtmlExporter {
    fn export_name() -> String {
        "HTML Document".to_string()
    }

    fn export_extension() -> String {
        ".html".to_string()
    }

    fn export_package(
        &mut self,
        package: &mut EvidencePackage,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut page = HtmlPage::new()
            .with_title(html_escape::encode_text(package.metadata().title()))
            .with_style(include_str!("html.css"))
            .with_script_literal(include_str!("html.js"));

        let title = HtmlElement::new(HtmlTag::Heading1)
            .with_raw(html_escape::encode_text(package.metadata().title()));
        page.add_html(title);

        let mut authors = String::new();
        for author in package.metadata().authors() {
            if let Some(email) = author.email() {
                authors.push_str(&format!("{} <{}>, ", author.name(), email));
            } else {
                authors.push_str(&format!("{}, ", author.name()));
            }
        }
        authors.pop();
        authors.pop();

        page.add_html(
            HtmlElement::new(HtmlTag::ParagraphText)
                .with_attribute("class", "authors")
                .with_raw(html_escape::encode_text(&authors)),
        );

        if let Some(description) = package.metadata().description() {
            page.add_html(
                HtmlElement::new(HtmlTag::ParagraphText)
                    .with_raw(html_escape::encode_text(description)),
            );
        }

        let mut test_cases: Vec<&TestCase> = package.test_case_iter()?.collect();
        test_cases.sort_by(|a, b| {
            a.metadata()
                .execution_datetime()
                .cmp(b.metadata().execution_datetime())
        });
        let mut first = true;
        let mut test_case_elems = vec![];
        let mut tab_container =
            HtmlElement::new(HtmlTag::UnorderedList).with_attribute("class", "tabs");
        for (idx, test_case) in test_cases.iter().enumerate() {
            let mut tab_elem = HtmlElement::new(HtmlTag::ListElement)
                .with_attribute("data-tab-index", idx)
                .with_link(format!("#tab{idx}"), test_case.metadata().title());
            if first {
                tab_elem.add_attribute("class", "selected");
            }
            tab_container.add_html(tab_elem);

            let elem = create_test_case_div(package.clone(), test_case)
                .map_err(crate::Error::OtherExportError)?
                .with_attribute("data-tab-index", idx)
                .with_attribute(
                    "class",
                    format!(
                        "tab-content {}",
                        if first {
                            first = false;
                            "selected"
                        } else {
                            ""
                        }
                    ),
                );
            test_case_elems.push(elem);
        }
        page.add_html(tab_container);
        for elem in test_case_elems {
            page.add_html(elem);
        }

        fs::write(path, page.to_html_string())?;

        Ok(())
    }

    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut page = HtmlPage::new()
            .with_title(html_escape::encode_text(package.metadata().title()))
            .with_style(include_str!("html.css"))
            .with_script_literal(include_str!("html.js"));

        let case = package
            .test_case(case)?
            .ok_or(crate::Error::OtherExportError(
                "Test case not found!".into(),
            ))?;
        let elem =
            create_test_case_div(package.clone(), case).map_err(crate::Error::OtherExportError)?;
        page.add_html(elem);

        fs::write(path, page.to_html_string())?;

        Ok(())
    }
}

/// Create the <div> element that holds a test case's data
fn create_test_case_div(
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<HtmlElement, Box<dyn std::error::Error>> {
    tracing::debug!("Creating HTML element for test case {}", test_case.id());
    let mut elem = HtmlElement::new(HtmlTag::Div)
        .with_html(
            HtmlElement::new(HtmlTag::Heading2)
                .with_attribute("class", "title")
                .with_raw(html_escape::encode_text(test_case.metadata().title())),
        )
        .with_html(
            HtmlElement::new(HtmlTag::ParagraphText)
                .with_attribute("class", "execution-time")
                .with_raw(test_case.metadata().execution_datetime().to_rfc2822()),
        );

    // Write evidence
    for evidence in test_case.evidence() {
        if let Some(caption) = evidence.caption() {
            elem.add_html(
                HtmlElement::new(HtmlTag::ParagraphText)
                    .with_attribute("class", "caption")
                    .with_raw(html_escape::encode_text(caption)),
            );
        }

        match evidence.kind() {
            EvidenceKind::Text => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                for line in text.lines() {
                    elem.add_html(
                        HtmlElement::new(HtmlTag::ParagraphText)
                            .with_raw(html_escape::encode_text(line)),
                    );
                }
            }
            EvidenceKind::RichText => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                if let Ok(rich_text) = parse_angelmark(&text) {
                    for line in rich_text {
                        match line {
                            AngelmarkLine::Newline => {
                                elem.add_html(HtmlElement::new(HtmlTag::LineBreak));
                            }
                            AngelmarkLine::Heading1(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading1),
                            )),
                            AngelmarkLine::Heading2(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading2),
                            )),
                            AngelmarkLine::Heading3(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading3),
                            )),
                            AngelmarkLine::Heading4(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading4),
                            )),
                            AngelmarkLine::Heading5(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading5),
                            )),
                            AngelmarkLine::Heading6(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Heading6),
                            )),
                            AngelmarkLine::TextLine(angelmark) => elem.add_html(angelmark_to_html(
                                &angelmark,
                                HtmlElement::new(HtmlTag::Span),
                            )),
                        }
                    }
                    elem.add_html(HtmlElement::new(HtmlTag::LineBreak));
                } else {
                    elem.add_html(HtmlElement::new(HtmlTag::CodeText).with_preformatted(text));
                }
            }
            EvidenceKind::Image => {
                let data = evidence.value().get_data(&mut package)?;
                let media = MediaFile::from(data);
                if let Some(mime) = media.mime_type() {
                    let data = base64::prelude::BASE64_STANDARD_NO_PAD.encode(media.data());
                    elem.add_html(
                        HtmlElement::new(HtmlTag::Image)
                            .with_attribute("src", format!("data:{mime};base64,{data}")),
                    );
                }
            }
            EvidenceKind::Http => {
                let data = evidence.value().get_data(&mut package)?;
                let data = String::from_utf8_lossy(data.as_slice());
                let data_parts = data
                    .split('\x1e')
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>();
                let request = data_parts.first().cloned().unwrap_or_default();
                let response = data_parts.get(1).cloned().unwrap_or_default();

                elem.add_html(
                    HtmlElement::new(HtmlTag::Div)
                        .with_attribute("class", "http-container")
                        .with_html(
                            HtmlElement::new(HtmlTag::Div)
                                .with_attribute("class", "http-request")
                                .with_html(
                                    HtmlElement::new(HtmlTag::CodeText)
                                        .with_preformatted(html_escape::encode_text(&request)),
                                ),
                        )
                        .with_html(
                            HtmlElement::new(HtmlTag::Div)
                                .with_attribute("class", "http-response")
                                .with_html(
                                    HtmlElement::new(HtmlTag::CodeText)
                                        .with_preformatted(html_escape::encode_text(&response)),
                                ),
                        ),
                );
            }
            EvidenceKind::File => {
                let data = evidence.value().get_data(&mut package)?;
                let data = base64::prelude::BASE64_STANDARD_NO_PAD.encode(data);
                let mime = if let EvidenceData::Media { hash } = evidence.value() {
                    if let Some(media) = package.get_media(hash).ok().flatten() {
                        if let Some(mime) = media.mime_type() {
                            mime.to_string()
                        } else {
                            "application/octet-stream".to_string()
                        }
                    } else {
                        "application/octet-stream".to_string()
                    }
                } else {
                    "application/octet-stream".to_string()
                };

                elem.add_html(
                    HtmlElement::new(HtmlTag::Div).with_html(
                        HtmlElement::new(HtmlTag::Link)
                            .with_attribute("href", format!("data:{mime};base64,{data}"))
                            .with_attribute(
                                "download",
                                evidence
                                    .original_filename()
                                    .clone()
                                    .unwrap_or(String::new()),
                            )
                            .with_raw(&if let Some(filename) = evidence.original_filename() {
                                filename.clone()
                            } else {
                                "Unnamed file".to_string()
                            }),
                    ),
                );
            }
        }
    }

    Ok(elem)
}

/// Convert Angelmark to HTML elements
fn angelmark_to_html(angelmark: &AngelmarkText, mut elem: HtmlElement) -> HtmlElement {
    match angelmark {
        AngelmarkText::Raw(txt) => elem.with_raw(html_escape::encode_text(txt)),
        AngelmarkText::Bold(content) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-bold");
            } else {
                elem.add_attribute("class", "richtext-bold");
            }
            angelmark_to_html(content, elem)
        }
        AngelmarkText::Italic(content) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-italic");
            } else {
                elem.add_attribute("class", "richtext-italic");
            }
            angelmark_to_html(content, elem)
        }
        AngelmarkText::Monospace(content) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-monospace");
            } else {
                elem.add_attribute("class", "richtext-monospace");
            }
            angelmark_to_html(content, elem)
        }
    }
}

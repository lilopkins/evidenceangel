use std::fmt::Write;
use std::fs;

use angelmark::{AngelmarkLine, AngelmarkTableAlignment, AngelmarkText, parse_angelmark};
use base64::Engine;
use build_html::{Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag};
use uuid::Uuid;

use crate::{EvidenceData, EvidenceKind, EvidencePackage, MediaFile, TestCase, TestCasePassStatus};

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
            .with_script_literal(include_str!("html.js"))
            .with_stylesheet(
                "https://unpkg.com/@highlightjs/cdn-assets@11.11.1/styles/default.min.css",
            )
            .with_script_link("https://unpkg.com/@highlightjs/cdn-assets@11.11.1/highlight.min.js")
            .with_script_link(
                "https://unpkg.com/@highlightjs/cdn-assets@11.11.1/languages/http.min.js",
            );

        let title = HtmlElement::new(HtmlTag::Heading1)
            .with_raw(html_escape::encode_text(package.metadata().title()));
        page.add_html(title);

        let mut authors = String::new();
        for author in package.metadata().authors() {
            if let Some(email) = author.email() {
                // SAFETY: This won't fail as there's no I/O
                write!(authors, "{} <{}>, ", author.name(), email).unwrap();
            } else {
                // SAFETY: This won't fail as there's no I/O
                write!(authors, "{}, ", author.name()).unwrap();
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

        if let Ok(branding_img) = std::env::var("EA_BRAND_IMAGE") {
            let data = fs::read(branding_img).map_err(|e| {
                std::io::Error::other(format!("Failed to read company brand image: {e}"))
            })?;
            let src = format!(
                "data:application/octet-stream;base64,{}",
                base64::prelude::BASE64_STANDARD_NO_PAD.encode(data)
            );
            page.add_image(
                src,
                format!(
                    "{} Logo",
                    std::env::var("EA_BRAND_NAME").unwrap_or("Brand".to_string())
                ),
            );
        }

        let test_cases: Vec<&TestCase> = package.test_case_iter()?.collect();
        let mut first = true;
        let mut test_case_elems = vec![];
        let mut tab_container =
            HtmlElement::new(HtmlTag::UnorderedList).with_attribute("class", "tabs");
        for (idx, test_case) in test_cases.iter().enumerate() {
            let mut tab_elem = HtmlElement::new(HtmlTag::ListElement)
                .with_attribute("data-tab-index", idx)
                .with_link(
                    format!("#tab{idx}"),
                    format!(
                        "{}{}",
                        match test_case.metadata().passed() {
                            None => "",
                            Some(TestCasePassStatus::Pass) => "✅&nbsp;",
                            Some(TestCasePassStatus::Fail) => "❌&nbsp;",
                        },
                        test_case.metadata().title()
                    ),
                );
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
        page.add_raw(r#"<label class="print-hide"><input type="checkbox" id="showAll" />&nbsp;Show all cases in one page</label>"#);
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
    let mut elem = HtmlElement::new(HtmlTag::Div);
    let mut meta_elem = HtmlElement::new(HtmlTag::Div)
        .with_attribute("class", "metadata")
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
    match test_case.metadata().passed() {
        None => (),
        Some(s) => {
            let s = match s {
                TestCasePassStatus::Pass => "✅ Pass",
                TestCasePassStatus::Fail => "❌ Fail",
            };
            meta_elem.add_html(
                HtmlElement::new(HtmlTag::ParagraphText)
                    .with_attribute("class", "status")
                    .with_raw(s),
            );
        }
    }
    if let Some(fields) = test_case.metadata().custom() {
        let mut dl = HtmlElement::new(HtmlTag::DescriptionList)
            .with_attribute("class", "custom-metadata-fields");
        for (key, value) in fields {
            let field = package
                .metadata()
                .custom_test_case_metadata()
                .as_ref()
                // SAFETY: guanteed by EVP spec
                .unwrap()
                .get(key)
                // SAFETY: guanteed by EVP spec
                .unwrap();
            dl.add_html(
                HtmlElement::new(HtmlTag::DescriptionListTerm)
                    .with_attribute("class", "custom-metadata-field-name")
                    .with_raw(field.name()),
            );
            dl.add_html(
                HtmlElement::new(HtmlTag::DescriptionListDescription)
                    .with_attribute("class", "custom-metadata-field-value")
                    .with_raw(value),
            );
        }
        meta_elem.add_html(dl);
    }
    elem.add_html(meta_elem);

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
                            AngelmarkLine::Newline(_span) => {
                                elem.add_html(HtmlElement::new(HtmlTag::LineBreak));
                            }
                            AngelmarkLine::Heading1(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading1);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::Heading2(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading2);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::Heading3(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading3);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::Heading4(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading4);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::Heading5(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading5);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::Heading6(angelmark_texts, _span) => {
                                let mut h = HtmlElement::new(HtmlTag::Heading6);
                                for angelmark in angelmark_texts {
                                    h.add_html(angelmark_to_html(
                                        &angelmark,
                                        HtmlElement::new(HtmlTag::Span),
                                    ));
                                }
                                elem.add_html(h);
                            }
                            AngelmarkLine::TextLine(angelmark, _span) => elem.add_html(
                                angelmark_to_html(&angelmark, HtmlElement::new(HtmlTag::Span)),
                            ),
                            AngelmarkLine::Table(table, _span) => {
                                let mut t = HtmlElement::new(HtmlTag::Table);
                                for row in table.rows() {
                                    let mut r = HtmlElement::new(HtmlTag::TableRow);
                                    for (col, cell) in row.cells().iter().enumerate() {
                                        let align =
                                            table.alignment().column_alignments()[col].alignment();
                                        let mut d = HtmlElement::new(HtmlTag::TableCell)
                                            .with_attribute(
                                                "style",
                                                format!(
                                                    "text-align:{}",
                                                    match align {
                                                        AngelmarkTableAlignment::Left => "left",
                                                        AngelmarkTableAlignment::Center => "center",
                                                        AngelmarkTableAlignment::Right => "right",
                                                    }
                                                ),
                                            );
                                        for angelmark in cell.content() {
                                            d.add_html(angelmark_to_html(
                                                angelmark,
                                                HtmlElement::new(HtmlTag::Span),
                                            ));
                                        }
                                        r.add_html(d);
                                    }
                                    t.add_html(r);
                                }
                                elem.add_html(t);
                            }
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
                                    HtmlElement::new(HtmlTag::PreformattedText).with_html(
                                        HtmlElement::new(HtmlTag::CodeText)
                                            .with_attribute("class", "language-http")
                                            .with_raw(html_escape::encode_text(&request)),
                                    ),
                                ),
                        )
                        .with_html(
                            HtmlElement::new(HtmlTag::Div)
                                .with_attribute("class", "http-response")
                                .with_html(
                                    HtmlElement::new(HtmlTag::PreformattedText).with_html(
                                        HtmlElement::new(HtmlTag::CodeText)
                                            .with_attribute("class", "language-http")
                                            .with_raw(html_escape::encode_text(&response)),
                                    ),
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
        AngelmarkText::Raw(txt, _span) => elem.with_raw(html_escape::encode_text(txt)),
        AngelmarkText::Bold(content, _span) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-bold");
            } else {
                elem.add_attribute("class", "richtext-bold");
            }
            angelmark_to_html(content, elem)
        }
        AngelmarkText::Italic(content, _span) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-italic");
            } else {
                elem.add_attribute("class", "richtext-italic");
            }
            angelmark_to_html(content, elem)
        }
        AngelmarkText::Monospace(content, _span) => {
            if let Some((_k, v)) = elem.attributes.iter_mut().find(|(k, _v)| k == "class") {
                v.push_str(" richtext-monospace");
            } else {
                elem.add_attribute("class", "richtext-monospace");
            }
            angelmark_to_html(content, elem)
        }
    }
}

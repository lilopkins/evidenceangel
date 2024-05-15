use std::fs;

use base64::Engine;
use domrs::{
    css_num, div, h1, h2, px, CssBorder, CssBorderStyle, CssColor, CssDocument, CssFontFamily,
    CssFontGenericFamily, CssNumber, CssProperty, CssRuleset, CssSelector, CssUnit, CssValue,
    HtmlBodyElement, HtmlDocument, HtmlElement, HtmlHeadElement, HtmlStyleElement,
};
use uuid::Uuid;

use crate::{EvidenceKind, EvidencePackage, MediaFile, TestCase};

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
        let mut body = HtmlBodyElement::new();

        let title = h1!(package.metadata().title());
        body.add_child(title);

        let mut authors = String::new();
        for author in package.metadata().authors() {
            if let Some(email) = author.email() {
                authors.push_str(&format!("{} &lt;{}&gt;, ", author.name(), email));
            } else {
                authors.push_str(&format!("{}, ", author.name()));
            }
        }
        authors.pop();
        authors.pop();

        body.add_child(HtmlElement::new("p").child(HtmlElement::new("em").content(&authors)));

        for test_case in package.test_case_iter()? {
            let elem = create_test_case_div(package.clone(), test_case)
                .map_err(crate::Error::OtherExportError)?;
            body.add_child(elem);
        }

        let document = HtmlDocument::new().head(
            HtmlHeadElement::default()
                .charset("utf-8")
                .title(package.metadata().title())
                .style(HtmlStyleElement::new(get_style())),
        );
        fs::write(path, document.body(body).to_string())?;

        Ok(())
    }

    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut body = HtmlBodyElement::new();

        let case = package
            .test_case(case)?
            .ok_or(crate::Error::OtherExportError(
                "Test case not found!".into(),
            ))?;
        let elem =
            create_test_case_div(package.clone(), case).map_err(crate::Error::OtherExportError)?;
        body.add_child(elem);

        let document = HtmlDocument::new().head(
            HtmlHeadElement::default()
                .charset("utf-8")
                .title(package.metadata().title())
                .style(HtmlStyleElement::new(get_style())),
        );
        fs::write(path, document.body(body).to_string())?;

        Ok(())
    }
}

fn create_test_case_div(
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<HtmlElement, Box<dyn std::error::Error>> {
    log::debug!("Creating HTML element for test case {}", test_case.id());
    let mut elem = div!().no_indent();
    elem.add_child(HtmlElement::new("hr"));
    elem.add_child(h2!(test_case.metadata().title()));
    elem.add_child(HtmlElement::new("p").child(
        HtmlElement::new("em").content(&test_case.metadata().execution_datetime().to_rfc2822()),
    ));

    // Write evidence
    for evidence in test_case.evidence() {
        if let Some(caption) = evidence.caption() {
            elem.add_child(HtmlElement::new("p").child(HtmlElement::new("em").content(caption)));
        }

        match evidence.kind() {
            EvidenceKind::Text => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                for line in text.lines() {
                    elem.add_child(HtmlElement::new("p").content(line));
                }
            }
            EvidenceKind::Image => {
                let data = evidence.value().get_data(&mut package)?;
                let media = MediaFile::from(data);
                if let Some(mime) = media.mime_type() {
                    let data = base64::prelude::BASE64_STANDARD_NO_PAD.encode(media.data());
                    elem.add_child(
                        HtmlElement::new("img")
                            .attribute("src", format!("data:{mime};base64,{data}")),
                    )
                }
            }
            EvidenceKind::Http => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                elem.add_child(
                    HtmlElement::new("code")
                        .no_indent()
                        .child(HtmlElement::new("pre").no_indent().content(&text)),
                );
            }
            EvidenceKind::File => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                elem.add_child(
                    HtmlElement::new("code")
                        .no_indent()
                        .child(HtmlElement::new("pre").no_indent().content(&text)),
                );
            }
        }
    }

    Ok(elem)
}

fn get_style() -> CssDocument {
    let mut style = CssDocument::new();
    style.add_element(domrs::CssElement::Ruleset(
        CssRuleset::new(CssSelector::new().element("body"))
            .declaration(
                CssProperty::FontFamily,
                CssValue::FontFamily(CssFontFamily::new(
                    &["Segoe UI".to_string(), "Liberation Sans".to_string()],
                    CssFontGenericFamily::SansSerif,
                )),
            )
            .declaration(CssProperty::MaxWidth, 800)
            .declaration(
                CssProperty::Margin,
                CssValue::Num2(css_num!(8., 0, CssUnit::Px), css_num!(0., 0, CssUnit::Auto)),
            ),
    ));
    style.add_element(domrs::CssElement::Ruleset(
        CssRuleset::new(CssSelector::new().element("img")).declaration(
            CssProperty::MaxWidth,
            CssValue::Num1(css_num!(100., 0, CssUnit::Perc)),
        ),
    ));
    style.add_element(domrs::CssElement::Ruleset(
        CssRuleset::new(CssSelector::new().element("hr"))
            .declaration(CssProperty::Border, CssValue::None)
            .declaration(
                CssProperty::BorderBottom,
                CssValue::Border(CssBorder::new(
                    px!(1),
                    CssBorderStyle::Solid,
                    CssColor::Black,
                )),
            ),
    ));
    style.add_element(domrs::CssElement::Ruleset(
        CssRuleset::new(CssSelector::new().element("pre")).declaration(
            CssProperty::BorderLeft,
            CssValue::Border(CssBorder::new(
                px!(1),
                CssBorderStyle::Solid,
                CssColor::Gray,
            )),
        ),
    ));
    style
}

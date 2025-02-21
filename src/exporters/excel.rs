use angelmark::{parse_angelmark, AngelmarkLine, AngelmarkText};
use rust_xlsxwriter::{Format, FormatBorder, Image, Workbook, Worksheet};
use uuid::Uuid;

use crate::{EvidenceKind, EvidencePackage, TestCase};

use super::Exporter;

/// An exporter to an Excel document.
#[derive(Default)]
pub struct ExcelExporter;

impl Exporter for ExcelExporter {
    fn export_name() -> String {
        "Excel Workbook".to_string()
    }

    fn export_extension() -> String {
        ".xlsx".to_string()
    }

    fn export_package(
        &mut self,
        package: &mut EvidencePackage,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut workbook = Workbook::new();
        workbook.read_only_recommended();

        // Create metadata sheet
        create_metadata_sheet(workbook.add_worksheet(), package)
            .map_err(crate::Error::OtherExportError)?;

        let test_cases: Vec<&TestCase> = package.test_case_iter()?.collect();
        for test_case in test_cases {
            let worksheet = workbook.add_worksheet();
            create_test_case_sheet(worksheet, package.clone(), test_case)
                .map_err(crate::Error::OtherExportError)?;
        }

        workbook
            .save(path)
            .map_err(|e| crate::Error::OtherExportError(e.into()))?;

        Ok(())
    }

    fn export_case(
        &mut self,
        package: &mut EvidencePackage,
        case: Uuid,
        path: std::path::PathBuf,
    ) -> crate::Result<()> {
        let mut workbook = Workbook::new();

        let worksheet = workbook.add_worksheet();
        let case = package
            .test_case(case)?
            .ok_or(crate::Error::OtherExportError(
                "Test case not found!".into(),
            ))?;
        create_test_case_sheet(worksheet, package.clone(), case)
            .map_err(crate::Error::OtherExportError)?;

        workbook
            .save(path)
            .map_err(|e| crate::Error::OtherExportError(e.into()))?;

        Ok(())
    }
}

/// Create the worksheet for the metadata
fn create_metadata_sheet(
    worksheet: &mut Worksheet,
    package: &EvidencePackage,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("Creating excel sheet for metadata");
    worksheet.set_name(package.metadata().title())?;
    worksheet.set_screen_gridlines(false);
    worksheet.set_column_width(0, 3)?; // To appear tidy

    let mut row = 1;

    let title = Format::new().set_bold().set_font_size(14);
    let italic = Format::new().set_italic();

    // Write title and execution timestamp
    worksheet.write_string_with_format(row, 1, package.metadata().title(), &title)?;
    row += 1;

    for author in package.metadata().authors() {
        row += 1;
        worksheet.write_string_with_format(row, 1, format!("{author}"), &italic)?;
    }

    row += 2;
    if let Some(description) = package.metadata().description() {
        worksheet.write_string(row, 1, description)?;
    }

    Ok(())
}

/// Create the worksheet that holds the test case's information
fn create_test_case_sheet(
    worksheet: &mut Worksheet,
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("Creating excel sheet for test case {}", test_case.id());
    worksheet.set_name(test_case.metadata().title())?;
    worksheet.set_screen_gridlines(false);
    worksheet.set_column_width(0, 3)?; // To appear tidy
    worksheet.set_column_width(1, 13)?; // For "Executed at:"
    worksheet.set_column_width(2, 20)?; // For execution date/time

    let mut row = 1;

    let title = Format::new().set_bold().set_font_size(14);
    let bold = Format::new().set_bold();
    let italic = Format::new().set_italic();
    let file_data = Format::new()
        .set_font_name("Courier New")
        .set_border_left(FormatBorder::Thick);

    // Write title and execution timestamp
    worksheet.write_string_with_format(row, 1, test_case.metadata().title(), &title)?;
    row += 1;
    worksheet.write(row, 1, "Executed at:")?;
    worksheet.write_with_format(
        row,
        2,
        &test_case.metadata().execution_datetime().naive_local(),
        &Format::new().set_num_format("yyyy-mm-dd hh:mm"),
    )?;
    row += 2;

    // Write evidence
    for evidence in test_case.evidence() {
        if let Some(caption) = evidence.caption() {
            worksheet.write_with_format(row, 1, caption, &italic)?;
            row += 1;
        }

        match evidence.kind() {
            EvidenceKind::Text => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                for line in text.lines() {
                    worksheet.write_string(row, 1, line)?;
                    row += 1;
                }
            }
            EvidenceKind::RichText => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());

                if let Ok(mut rich_text) = parse_angelmark(&text) {
                    if !matches!(rich_text.last(), Some(AngelmarkLine::Newline(_))) {
                        rich_text.push(AngelmarkLine::Newline(angelmark::OwnedSpan::default()));
                    }
                    let mut line_buffer: Vec<(Format, String)> = vec![];
                    for line in rich_text {
                        match line {
                            AngelmarkLine::Newline(_span) => {
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &line_buffer
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                line_buffer.clear();
                                row += 1;
                            }
                            AngelmarkLine::Heading1(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(32),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::Heading2(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(28),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::Heading3(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(24),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::Heading4(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(18),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::Heading5(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(16),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::Heading6(angelmark, _span) => {
                                let fragments = angelmark
                                    .iter()
                                    .map(|text| {
                                        angelmark_to_excel(
                                            text,
                                            Format::default().set_font_size(14),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                worksheet.write_rich_string(
                                    row,
                                    1,
                                    &fragments
                                        .iter()
                                        .map(|(f, s)| (f, s.as_str()))
                                        .collect::<Vec<_>>(),
                                )?;
                                row += 1;
                            }
                            AngelmarkLine::TextLine(angelmark, _span) => {
                                line_buffer.push(angelmark_to_excel(&angelmark, Format::default()));
                            }
                        }
                    }
                } else {
                    for line in text.lines() {
                        worksheet.write_string_with_format(
                            row,
                            1,
                            line,
                            &Format::default().set_font_name("Courier New"),
                        )?;
                        row += 1;
                    }
                }
            }
            EvidenceKind::Image => {
                let data = evidence.value().get_data(&mut package)?;
                let image = Image::new_from_buffer(data.as_slice())?;
                worksheet.insert_image(row, 1, &image)?;

                // Calculate row offset
                let height_in = image.height() / image.height_dpi();
                let row_units_per_in = 4.87;
                let num_rows_to_skip = (height_in * row_units_per_in).ceil() as u32;
                row += num_rows_to_skip;
            }
            EvidenceKind::Http => {
                worksheet.write_string_with_format(row, 1, "HTTP Request", &bold)?;
                row += 1;
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());
                for line in text.lines() {
                    worksheet.write_string_with_format(row, 1, line, &file_data)?;
                    row += 1;
                }
            }
            EvidenceKind::File => {
                let data = evidence.value().get_data(&mut package)?;
                let text = String::from_utf8_lossy(data.as_slice());

                if let Some(filename) = evidence.original_filename() {
                    worksheet.write_string(row, 1, filename)?;
                    row += 1;
                }

                // Check if plaintext ASCII
                let mut is_printable = true;
                for c in text.chars() {
                    if !c.is_ascii() {
                        is_printable = false;
                        break;
                    }
                }

                if is_printable {
                    for line in text.lines() {
                        worksheet.write_string_with_format(row, 1, line, &file_data)?;
                        row += 1;
                    }
                } else {
                    worksheet.write_string_with_format(row, 1, "binary file data", &italic)?;
                    row += 1;
                }
            }
        }

        row += 1;
    }

    Ok(())
}

/// Convert Angelmark to Excel format data
fn angelmark_to_excel(angelmark: &AngelmarkText, format: Format) -> (Format, String) {
    match angelmark {
        AngelmarkText::Raw(txt, _span) => (format, txt.clone()),
        AngelmarkText::Bold(content, _span) => angelmark_to_excel(content, format.set_bold()),
        AngelmarkText::Italic(content, _span) => angelmark_to_excel(content, format.set_italic()),
        AngelmarkText::Monospace(content, _span) => {
            angelmark_to_excel(content, format.set_font_name("Courier New"))
        }
    }
}

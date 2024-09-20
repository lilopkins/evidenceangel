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

        // Create metadata sheet
        create_metadata_sheet(workbook.add_worksheet(), package)
            .map_err(crate::Error::OtherExportError)?;

        let mut test_cases: Vec<&TestCase> = package.test_case_iter()?.collect();
        test_cases.sort_by(|a, b| {
            a.metadata()
                .execution_datetime()
                .cmp(b.metadata().execution_datetime())
        });
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

fn create_metadata_sheet(
    worksheet: &mut Worksheet,
    package: &EvidencePackage,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Creating excel sheet for metadata");
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

    Ok(())
}

fn create_test_case_sheet(
    worksheet: &mut Worksheet,
    mut package: EvidencePackage,
    test_case: &TestCase,
) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Creating excel sheet for test case {}", test_case.id());
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
                for line in text.lines() {
                    worksheet.write_string_with_format(row, 1, line, &file_data)?;
                    row += 1;
                }
            }
        }

        row += 1;
    }

    Ok(())
}

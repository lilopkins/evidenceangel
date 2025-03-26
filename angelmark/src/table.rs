use std::cmp::Ordering;

use getset::Getters;
use pest::iterators::Pair;

use crate::{lexer::Rule, AngelmarkText, EqIgnoringSpan, OwnedSpan};

/// A table
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct AngelmarkTable {
    /// The rows of the table
    pub(crate) rows: Vec<AngelmarkTableRow>,
    /// Table width
    pub(crate) width: usize,
    /// The alignment row of the table
    pub(crate) alignment: AngelmarkTableAlignmentRow,
    /// The row's full span
    pub(crate) span: OwnedSpan,
}

impl AngelmarkTable {
    /// Get the width and height of this table
    #[must_use]
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.rows.len())
    }

    /// Get the width in letters of a particular column
    #[must_use]
    pub fn column_width(&self, col: usize) -> usize {
        let mut max_width = 0;

        for row in &self.rows {
            if let Some(cell) = &row.cells.get(col) {
                max_width = max_width.max(cell.content().iter().map(get_text_width).sum());
            }
        }

        max_width
    }
}

fn get_text_width(text: &AngelmarkText) -> usize {
    match text {
        AngelmarkText::Bold(text, _span)
        | AngelmarkText::Italic(text, _span)
        | AngelmarkText::Monospace(text, _span) => get_text_width(text),
        AngelmarkText::Raw(text, _span) => text.len(),
    }
}

impl From<Pair<'_, Rule>> for AngelmarkTable {
    fn from(pair: Pair<'_, Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::Table);

        let mut rows = vec![];
        let mut width = None;
        let mut alignment_row = None;

        let span = pair.as_span();
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::TableRow => {
                    let mut row = AngelmarkTableRow::from(pair);
                    if let Some(target_width) = &width {
                        match row.cells().len().cmp(target_width) {
                            Ordering::Equal => (),
                            Ordering::Greater => {
                                tracing::warn!("More rows than expected width found!");
                            }
                            Ordering::Less => {
                                while row.cells.len() < *target_width {
                                    row.cells.push(AngelmarkTableCell {
                                        content: vec![],
                                        span: OwnedSpan::default(),
                                    });
                                }
                            }
                        }
                    } else {
                        width = Some(row.cells().len());
                    }
                    rows.push(row);
                }
                Rule::TableAlignmentRow => {
                    alignment_row = Some(AngelmarkTableAlignmentRow::from(pair));
                }
                _ => unreachable!(),
            }
        }

        Self {
            rows,
            width: width.unwrap(),
            alignment: alignment_row.unwrap(),
            span: span.into(),
        }
    }
}

impl EqIgnoringSpan for AngelmarkTable {
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        self.width == other.width
            && self.alignment.eq_ignoring_span(&other.alignment)
            && self
                .rows
                .iter()
                .zip(&other.rows)
                .all(|(a, b)| a.eq_ignoring_span(b))
    }
}

/// A table row
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct AngelmarkTableRow {
    /// The cells within the row
    pub(crate) cells: Vec<AngelmarkTableCell>,
    /// The span of the table row
    pub(crate) span: OwnedSpan,
}

impl From<Pair<'_, Rule>> for AngelmarkTableRow {
    fn from(value: Pair<'_, Rule>) -> Self {
        assert_eq!(value.as_rule(), Rule::TableRow);

        let span = value.as_span();
        let mut cells = vec![];
        for cell in value.into_inner() {
            cells.push(AngelmarkTableCell::from(cell));
        }

        Self {
            cells,
            span: span.into(),
        }
    }
}

impl EqIgnoringSpan for AngelmarkTableRow {
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        self.cells.len() == other.cells.len()
            && self
                .cells
                .iter()
                .zip(&other.cells)
                .all(|(a, b)| a.eq_ignoring_span(b))
    }
}

/// A table alignment row
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct AngelmarkTableAlignmentRow {
    /// The alignment cells of each column
    pub(crate) column_alignments: Vec<AngelmarkTableAlignmentCell>,
    /// The span of the alignment row
    pub(crate) span: OwnedSpan,
}

impl From<Pair<'_, Rule>> for AngelmarkTableAlignmentRow {
    fn from(value: Pair<'_, Rule>) -> Self {
        assert_eq!(value.as_rule(), Rule::TableAlignmentRow);

        let span = value.as_span();
        let mut column_alignments = vec![];
        for cell in value.into_inner() {
            column_alignments.push(AngelmarkTableAlignmentCell::from(cell));
        }

        Self {
            column_alignments,
            span: span.into(),
        }
    }
}

impl EqIgnoringSpan for AngelmarkTableAlignmentRow {
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        self.column_alignments.len() == other.column_alignments.len()
            && self
                .column_alignments
                .iter()
                .zip(&other.column_alignments)
                .all(|(a, b)| a.eq_ignoring_span(b))
    }
}

/// A table alignment cell
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct AngelmarkTableAlignmentCell {
    /// The alignment specified by this alignment cell
    pub(crate) alignment: AngelmarkTableAlignment,
    /// The cell span
    pub(crate) span: OwnedSpan,
}

impl EqIgnoringSpan for AngelmarkTableAlignmentCell {
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        self.alignment == other.alignment
    }
}

impl From<Pair<'_, Rule>> for AngelmarkTableAlignmentCell {
    fn from(value: Pair<'_, Rule>) -> Self {
        assert_eq!(value.as_rule(), Rule::TableAlignmentCell);

        let s = value.as_str();
        let alignment = if s.starts_with(':') && s.ends_with(':') {
            AngelmarkTableAlignment::Center
        } else if s.starts_with(':') {
            AngelmarkTableAlignment::Left
        } else if s.ends_with(':') {
            AngelmarkTableAlignment::Right
        } else {
            // (default)
            AngelmarkTableAlignment::Left
        };

        Self {
            alignment,
            span: value.as_span().into(),
        }
    }
}

/// A specified alignment
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkTableAlignment {
    /// Align left
    Left,
    /// Align center
    Center,
    /// Align right
    Right,
}

/// A table cell
#[derive(Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct AngelmarkTableCell {
    /// The content of this cell
    pub(crate) content: Vec<AngelmarkText>,
    /// The span for this cell
    pub(crate) span: OwnedSpan,
}

impl From<Pair<'_, Rule>> for AngelmarkTableCell {
    fn from(value: Pair<'_, Rule>) -> Self {
        assert_eq!(value.as_rule(), Rule::TableCell);

        let span = value.as_span();
        let mut content = vec![];
        for pair in value.into_inner() {
            content.push(crate::parse_text_content(pair));
        }

        Self {
            content,
            span: span.into(),
        }
    }
}

impl EqIgnoringSpan for AngelmarkTableCell {
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        self.content.len() == other.content.len()
            && self
                .content
                .iter()
                .zip(&other.content)
                .all(|(a, b)| a.eq_ignoring_span(b))
    }
}

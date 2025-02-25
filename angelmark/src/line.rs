use crate::{AngelmarkTable, AngelmarkText, EqIgnoringSpan, OwnedSpan};

/// A line of markup in AngelMark
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkLine {
    /// A level 1 heading.
    Heading1(Vec<AngelmarkText>, OwnedSpan),
    /// A level 2 heading.
    Heading2(Vec<AngelmarkText>, OwnedSpan),
    /// A level 3 heading.
    Heading3(Vec<AngelmarkText>, OwnedSpan),
    /// A level 4 heading.
    Heading4(Vec<AngelmarkText>, OwnedSpan),
    /// A level 5 heading.
    Heading5(Vec<AngelmarkText>, OwnedSpan),
    /// A level 6 heading.
    Heading6(Vec<AngelmarkText>, OwnedSpan),
    /// A line of text.
    TextLine(AngelmarkText, OwnedSpan),
    /// A table.
    Table(AngelmarkTable, OwnedSpan),
    /// A line separator.
    Newline(OwnedSpan),
}

impl AngelmarkLine {
    /// Get the span from this line
    #[must_use]
    pub fn span(&self) -> &OwnedSpan {
        match self {
            Self::Heading1(_, span)
            | Self::Heading2(_, span)
            | Self::Heading3(_, span)
            | Self::Heading4(_, span)
            | Self::Heading5(_, span)
            | Self::Heading6(_, span)
            | Self::TextLine(_, span)
            | Self::Table(_, span)
            | Self::Newline(span) => span,
        }
    }
}

impl EqIgnoringSpan for AngelmarkLine {
    /// Compare two [`AngelmarkLine`] instances, ignoring their span.
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        match self {
            Self::Heading1(inner, _) => {
                if let Self::Heading1(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading2(inner, _) => {
                if let Self::Heading2(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading3(inner, _) => {
                if let Self::Heading3(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading4(inner, _) => {
                if let Self::Heading4(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading5(inner, _) => {
                if let Self::Heading5(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::Heading6(inner, _) => {
                if let Self::Heading6(other_inner, _) = other {
                    if inner.len() != other_inner.len() {
                        return false;
                    }
                    inner
                        .iter()
                        .zip(other_inner.iter())
                        .all(|(a, b)| a.eq_ignoring_span(b))
                } else {
                    false
                }
            }
            Self::TextLine(inner, _) => {
                if let Self::TextLine(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Table(inner, _) => {
                if let Self::Table(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Newline(_) => {
                matches!(other, Self::Newline(_))
            }
        }
    }
}

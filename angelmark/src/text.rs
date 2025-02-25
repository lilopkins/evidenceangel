use crate::{EqIgnoringSpan, OwnedSpan};

/// Textual content
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AngelmarkText {
    /// Raw text
    Raw(String, OwnedSpan),
    /// Bold content
    Bold(Box<AngelmarkText>, OwnedSpan),
    /// Italicised content
    Italic(Box<AngelmarkText>, OwnedSpan),
    /// Monospace content
    Monospace(Box<AngelmarkText>, OwnedSpan),
}

impl AngelmarkText {
    /// Get the span from this text
    #[must_use]
    pub fn span(&self) -> &OwnedSpan {
        match self {
            Self::Raw(_, span)
            | Self::Bold(_, span)
            | Self::Italic(_, span)
            | Self::Monospace(_, span) => span,
        }
    }
}

impl EqIgnoringSpan for AngelmarkText {
    /// Compare two [`AngelmarkText`] instances, ignoring their span.
    fn eq_ignoring_span(&self, other: &Self) -> bool {
        match self {
            Self::Raw(inner, _) => {
                if let Self::Raw(other_inner, _) = other {
                    inner == other_inner
                } else {
                    false
                }
            }
            Self::Bold(inner, _) => {
                if let Self::Bold(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Italic(inner, _) => {
                if let Self::Italic(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
            Self::Monospace(inner, _) => {
                if let Self::Monospace(other_inner, _) = other {
                    inner.eq_ignoring_span(other_inner)
                } else {
                    false
                }
            }
        }
    }
}

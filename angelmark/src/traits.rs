/// Check equality between two parsed structs ignoring the internal spans.
pub trait EqIgnoringSpan {
    /// Check equality between two parsed structs ignoring the internal spans.
    #[must_use]
    fn eq_ignoring_span(&self, other: &Self) -> bool;
}

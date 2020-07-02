//! Common types used by libslide.

/// Describes the character span of a substring in a text.
///
/// For example, in "abcdef", "bcd" has the span (1, 4).
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Span {
    /// Inclusive lower bound index of the span
    pub lo: usize,
    /// Exclusive upper bound index of the span
    pub hi: usize,
}

impl From<(usize, usize)> for Span {
    fn from(span: (usize, usize)) -> Self {
        Self {
            lo: span.0,
            hi: span.1,
        }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(span: std::ops::Range<usize>) -> Self {
        Self {
            lo: span.start,
            hi: span.end,
        }
    }
}

impl From<Span> for (usize, usize) {
    fn from(span: Span) -> Self {
        (span.lo, span.hi)
    }
}

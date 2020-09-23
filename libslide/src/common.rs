//! Common types used by libslide.

use std::cmp::Ordering;

/// Describes the character span of a substring in a text.
///
/// For example, in "abcdef", "bcd" has the span (1, 4).
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct Span {
    /// Inclusive lower bound index of the span, in terms of number of chars
    pub lo: usize,
    /// Exclusive upper bound index of the span, in terms of number of chars
    pub hi: usize,
}

/// A dummy span for use in places where a span is not (yet) known.
///
/// Clearly this span is incorrect since lo = 10001 > 1 = hi, but a well-formed span must observe
/// the invariant lo <= hi.
///
/// NB: This is only to be used during migration and refactoring. Do *not* use this for new
/// interned expressions.
pub(crate) static DUMMY_SP: Span = Span { lo: 10001, hi: 1 };

impl Span {
    pub(crate) fn to(&self, other: Span) -> Span {
        Self {
            lo: self.lo,
            hi: other.hi,
        }
    }

    pub(crate) fn over<'a>(&self, content: &'a str) -> &'a str {
        let mut indices = content.char_indices().map(|(i, _)| i);
        let lo = indices.nth(self.lo).unwrap();
        let hi = vec![lo]
            .into_iter()
            .chain(indices)
            .nth(self.hi - self.lo)
            .unwrap_or_else(|| content.len());
        &content[lo..hi]
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.lo < other.lo {
            Ordering::Less
        } else if self.lo > other.lo {
            Ordering::Greater
        } else if self.hi < other.hi {
            Ordering::Less
        } else if self.hi > other.hi {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
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

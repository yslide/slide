//! Module `ptr` provides a generic smart pointer interface for use in slide LS.

use std::sync::Arc;

/// A smart pointer to data on the heap.
// Currently Box is used, but later on it may be something else.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct P<T>(Arc<T>);

/// Creates a new [`P` pointer](P) to some data.
pub(crate) fn p<T>(item: T) -> P<T> {
    P(Arc::new(item))
}

impl<T> P<T> {
    /// Duplicates the pointer.
    pub fn dupe(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> std::ops::Deref for P<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> AsRef<T> for P<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

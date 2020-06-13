use std::collections::VecDeque;
use std::vec::IntoIter;

/// A [`TakeWhile`]-like struct that tests a predicate by peeking rather than consuming an iterator.
///
/// rustlib's [`TakeWhile`] consumes items in an iterator until its predicate is no longer satisfied.
/// This means that the first item that fails the predicate will also be consumed. For example,
///
/// ```rust
/// let nums = vec![1, 2, 3, 4, 5];
/// let mut iter = nums.iter();
/// let less_than_4: Vec<usize> = iter.by_ref().take_while(|n| **n < 4).cloned().collect();
/// assert_eq!(less_than_4, &[1, 2, 3]);
/// assert_eq!(iter.next(), Some(&5)); // 4 was consumed!
/// ```
///
/// `PeekingTakeWhile` implements a [`TakeWhile`]-like functionality without consuming items that fail
/// its predicate.
///
/// TODO: Ideally a `PeekingTakeWhile` would take a [`Peekable`] trait object rather than a
/// `PeekIter`, but rustlib doesn't provide a [`Peekable`] trait yet. See the [Pre-RFC].
///
/// [`TakeWhile`]: core::iter::TakeWhile
/// [`Peekable`]: core::iter::Peekable
/// [Pre-RFC]: https://internals.rust-lang.org/t/pre-rfc-make-peekable-trait-for-iterator
struct PeekingTakeWhile<'a, T, P>
where
    T: Clone + 'a,
    P: Fn(&T) -> bool,
{
    /// A mutable reference to the underlying iterator is taken because we actually do want to
    /// consume items that match the predicate.
    peeker: &'a mut PeekIter<T>,
    predicate: P,
}

impl<'a, T, P> Iterator for PeekingTakeWhile<'a, T, P>
where
    T: Clone + 'a,
    P: Fn(&T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if let Some(v) = self.peeker.peek() {
            if (self.predicate)(&v) {
                return self.peeker.next();
            }
        }
        None
    }
}

/// An iterator that supports arbitrary-length peeking.
///
/// This struct is a beefed-up version of rustlib's [`Peekable`], which supports only peeking at the
/// next item in an iterator. Multi-length peeks may be required by applications that need to
/// establish a context; for example, a parser.
///
/// [`Peekable`]: core::iter::Peekable
pub struct PeekIter<T>
where
    T: Clone,
{
    iter: IntoIter<T>,
    /// A store of items we had to consume from the iterator for peeking.
    lookahead: VecDeque<Option<T>>,
}

impl<T> PeekIter<T>
where
    T: Clone,
{
    pub fn new(iter: IntoIter<T>) -> Self {
        let mut lookahead = VecDeque::new();
        lookahead.reserve(5); // optimistically we won't be peeking more than this

        Self { iter, lookahead }
    }

    /// Returns a reference to the next value in the iterator, without consuming it, or `None` if
    /// the iteration is complete.
    pub fn peek(&mut self) -> Option<&T> {
        if self.lookahead.is_empty() {
            // Hopefully the branch gets optimized out. Not sure if we can reduce it.
            let next = self.iter.next();
            self.lookahead.push_back(next);
        }
        self.lookahead[0].as_ref()
    }

    /// Returns a deque of up to `n` peeked items mapped over a function `f`.
    ///
    /// The length of the returned deque is `n` or the number of items remaining in the iteration,
    /// whichever is lower.
    pub fn peek_map_n<R>(&mut self, n: usize, f: fn(&T) -> R) -> VecDeque<R> {
        while self.lookahead.len() < n {
            let next = self.iter.next();
            self.lookahead.push_back(next);
        }
        self.lookahead
            .iter()
            .take(n)
            .filter_map(|o| o.as_ref())
            .map(f)
            .collect()
    }

    /// Adds an item to the front of the current iteration.
    pub fn push_front(&mut self, item: T) {
        self.lookahead.push_front(Some(item));
    }
}

impl<T> Iterator for PeekIter<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.lookahead
            .pop_front()
            // Note that unwrap_or *cannot* be used here because it is easily evaluated, and will
            // evaluate `self.iter.next()` before the lookahead is checked!
            .unwrap_or_else(|| self.iter.next())
    }
}

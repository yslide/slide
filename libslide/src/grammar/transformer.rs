use super::Grammar;

/// A trait for transforming one grammar into another.
/// This transformer takes ownership of the grammar it transforms.
pub trait Transformer<T: Grammar, U: Grammar> {
    fn transform(&self, item: T) -> U;
}

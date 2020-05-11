use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn hash<T: Hash>(expr: &T) -> u64 {
    // There is no way to reset a hasher's state, so we create a new one each time.
    let mut hasher = DefaultHasher::new();
    expr.hash(&mut hasher);
    hasher.finish()
}

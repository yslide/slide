//! A collection of algorithms used by [evaluation rules][evaluation rules].
//!
//! This module is decoupled from [evaluation rules] and the [partial evaluator] because:
//!
//! - `math`'s algorithms often use representations differing from the libslide
//!   [grammar][grammar], for which transforming shims are required.
//! - `math` can be developed independently from `libslide`, with the goal of eventual use in
//!   evaluator rules.
//!
//! [evaluation rules]: crate::evaluator_rules
//! [grammar]: crate::grammar
//! [partial_evaluator]: crate::partial_evaluator

mod fraction;

mod gcd;
pub use gcd::*;

mod poly;
pub use poly::*;

mod gcd_poly_zz;
pub use gcd_poly_zz::*;

//! [Grammar]->[Grammar] transforming rules, used by the [partial_evaluator].
//!
//! [Grammar]: crate::Grammar
//! [partial_evaluator]: crate::partial_evaluator

mod pattern_match;
mod registry;
mod rule;
mod unbuilt_rule;

pub use registry::RuleName;
pub use registry::RuleSet;

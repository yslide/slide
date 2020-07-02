//! libslide is the core of slide, implementing the end-to-end processing of a slide program.
//!
//! slide is described as a "static expression optimizer", which can be thought of as rougly equivalent
//! to an "expression simplifier". Both of these terms are ambiguous ideas reasonable people can
//! disagree on. For example, depending on the evaluation context, either `x ^ 2` or `x * x` may be
//! "optimized" form of the same expression.
//!
//! slide uses reasonable (operation-reducing) simplification rules to achieve this goal. Recognizing the
//! ambiguities described above, slide also hopes to make optimization customizable on the side of the
//! user.
//!
//! This isn't the only design goal of slide; others include
//!
//! - Simplification as a platform, where optimization rules are configurable plugins.
//! - Strong support for environments slide is most used in (e.g. LaTeX and ad-hoc interactive queries).
//! - Easy integration with text editors and language servers.
//! - Evaluation of mathematical statements even in ambiguous contexts.
//!
//! slide is still far from being complete. Contributions of any kind are warmly welcomed, as they help
//! us progress in the achievement of these goals.
//!
//! The rest of this document discusses the architecture of slide.
//!
//! ## A brief overview of slide's architecture
//!
//! <img src="https://docs.google.com/drawings/d/e/2PACX-1vSguCnT1JCmJeF3NG7o1VYhp8Pqo4Qn093ysYcRRIR_KRVZWbrTALkD2pRPZRqLCZpxvSyuWraZFaUk/pub?w=864&amp;h=432">
//!
//! slide's design is very similar to a compiler front end, sans the type checking and ups the partial
//! evaluation.
//!
//! The parser is a simple, hand-written RD parser designed to produce nice error messages. The parser
//! module supports both expressions and [expression patterns](#expression-patterns).
//!
//! Given a parsed expression, the evaluator loops application of [string](#string-rules) and
//! [function rules](#function-rules) until no rule further reduces the expression. This is the most
//! interesting part of slide, which we discuss in the next section.
//!
//! Finally, the simplified expression is emitted. Currently slide supports a few emission strategies,
//! and we are interested in adding more.
//!
//! ## Evaluator
//!
//! The `partial_evaluator` module loops the application of simplification rules on an expression until
//! a rule no longer reduces an expression.
//!
//! ### Kinds of simplification rules
//!
//! slide supports "string" and "function" rules. String rules are [expression
//! patterns](#expression-patterns) mappings that describe how the rule should be applied to an
//! expression. For example, the string rule
//!
//! ```slide
//! 1 * _a -> _a
//! ```
//!
//! says "1 times any expression `a` yields the expression `a`". String rules are applied to an
//! expression by pattern matching the ASTs of the LHS of the string rule and the expression to be
//! simplified. If the matching is successful, the matches are substituted on the RHS of the string
//! rule. As an example, we apply the above rule on several expressions:
//!
//! ```slide
//! 1 * (2 + v) -> (2 + v)
//! 1 + (2 + v) -> no match!
//! (2 + v) * 1 -> no match!
//! ```
//!
//! The fact that the third expression fails to match with the rule demonstrates one weakness of string
//! rules. Another rule, `_a * 1 -> _a`, is needed to represent this case. (In practice, slide tries to
//! permute the commutativity of an expression to induce the application of string rules written in a
//! particular way, but this is not perfect).
//!
//! String rules also have no way to represent evaluation. Since they are purely mappings of terms,
//! there is no way to describe the operation of addition with a string rule.
//!
//! Because of these limitations in string rules, slide also supports function rules. These are
//! functions of the form
//!
//! ```ignore
//! fn(expr: Expr) -> Option<Expr>
//! ```
//!
//! Function rules are much more powerful because they have access to an entire expression tree and can
//! perform evaluations, but are responsible for their own pattern matching and more difficult to
//! prove correctness for.
//!
//! As a summary, string rules are easy to write and inject into a slide program, but are very limited
//! in their application. Function rules are more difficult to write and prove correct, but are much
//! more flexible.
//!
//! #### Function rule evaluation modules
//!
//! slide-internal function rules may use a number of other modules to aid evaluation. We list the most
//! common ones here:
//!
//! - `math`: the math module is a collection of algorithms used in the evaluation of an expression, like
//!     polynomial division. This module is often developed independently with the goal of eventual use
//!     in slide rules. The math module provides shims for converting between [expression representation](#expression-representation)
//!     and the data representations used by the math module's algorithms.
//! - `flatten/fold`: this module tries to [normalize](#normalization) an expression with postconditions
//!     rules can rely on in their evaluation.
//! - `roll/unroll`: these are utility functions that unroll an expression into a list of terms, or roll
//!     a list up into an expression AST.
//!
//! ### Normalization
//!
//! It is useful to normalize expressions in to a consistent form that can be expected by function rules
//! (and to some extent string rules). The `flatten/fold` module provides expression normalization like
//! combining like terms, normalizing similar operations (e.g. subtractions become additions), and other
//! procedures whose postconditions rules can then rely on.
//!
//! Normalization is optional, and should be disabled when a user disables simplification rules used in
//! the normalization process.
//!
//! ## Expression Representation
//!
//! Expressions are primarily represented as interned ASTs. For example, the expression "1 + 1" parses
//! to an AST that looks like something like
//!
//! ```slide
//! (+ <1>@common-1 <1>@common-1)
//! ```
//!
//! where `<1>@common-1` is the expression `1` held at the example pointer address `common-1`.
//!
//! The point here is that the common expression `1` is eliminated, and both references to `1` point to
//! the same underlying expression.
//!
//! Because expressions are already held in boxed AST nodes, this interning process does not take an
//! additional memory hit and provides several advantages, like skipping evaluation passes on identical
//! expressions.
//!
//! ## Glossary
//!
//! #### String rules
//!
//! Simplification rules that describe the mapping of an [expression pattern](#expression-patterns) in a
//! string form. For example,
//!
//! ```slide
//! 1 * _a -> _a
//! ```
//!
//! is a string rule that describes the simplification "1 times any expression `a` yields the expression
//! `a`".
//!
//! String rules are easy to write, but are strict in what they match and cannot express evaluation.
//!
//! #### Function rules
//!
//! Function rules are functions with the signature
//!
//! ```ignore
//! fn(expr: Expr) -> Option<Expr>
//! ```
//!
//! Function rules are given an expression and can perform arbitrary evaluation to try to simplify the
//! expression. Function rules are responsible for matching the expression themselves.
//!
//! Function rules are more difficult to write and prove correct, but are very flexible.
//!
//! #### Expressions
//!
//! Expressions are the inputs to a slide instance. For example, given the expression
//!
//! ```slide
//! 2x + 3 + 4 + x
//! ```
//!
//! slide should emit the lowered expression `3x + 7`.
//!
//! #### Expression Patterns
//!
//! Expression patterns describe the shape of a expression. Expression patterns are used by [string rules](#string-rules)
//! to describe how a rule should be applied.
//!
//! The syntax of expressions and expression patterns is nearly identical, but expression patterns have
//! pattern metavariables rather than variables. The following patterns are available:
//!
//! | pattern   | matches        |
//! |:--------- |:-------------- |
//! | `#<name>` | A constant     |
//! | `$<name>` | A variable     |
//! | `_<name>` | Any expression |
//!
//! A metavariable can match exactly one expression. This means that if `_a` matches `1 + 2`, all other
//! references to `_a` must match `1 + 2` as well.
//!
//! Some examples of expression patterns and matching expressions:
//!
//! | expression pattern | matches |
//! | -- | -- |
//! | `1 + #a` | `1 + 5.2` |
//! | `#a + $a` | `10 + v` |
//! | `_a / $a * _a` | `(9 + 1) / v * (9 + 1)` |
//! | `_a + _b * $c * #d` | `(x ^ 2) + (5 / y) * w * 15` |

#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://avatars1.githubusercontent.com/u/49662722?s=400&u=62119505c71017e88a2728f7a1257b3506481441&v=4"
)]

mod common;
pub use common::*;

pub mod diagnostics;

pub mod scanner;
pub use scanner::scan;

mod parser;
pub use parser::parse_expression;
pub use parser::parse_expression_pattern;

mod partial_evaluator;
pub use partial_evaluator::evaluate;
pub use partial_evaluator::EvaluatorContext;

mod evaluator_rules;
mod grammar;
pub use grammar::Grammar;

mod math;

#[cfg(feature = "benchmark-internals")]
pub use math::*;

mod utils;

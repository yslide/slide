//! Diagnostic errors produced by the parser.

use crate::diagnostics::{DiagnosticRecord, DiagnosticRegistry};

macro_rules! define_errors {
    ($($(#[doc = $doc:expr])+ $code:ident: $error:ident $gen_macro:tt)*) => {$(
        $(#[doc = $doc])+
        pub(crate) struct $error;

        impl DiagnosticRecord for $error {
            const CODE: &'static str = stringify!($code);
            const EXPLANATION: &'static str = concat!($($doc, "\n"),+);
        })*

        pub struct ParseErrors;

        impl DiagnosticRegistry for ParseErrors {
            fn codes_with_explanations() -> Vec<(&'static str, &'static str)> {
                let mut vec = Vec::new();
                $(vec.push(($error::CODE, $error::EXPLANATION));)*
                vec
            }
        }

        $(
            macro_rules! $error $gen_macro
        )*
    };
}

define_errors! {
    ///This error fires on tokens that are not connected to the rest of a primary statement in a
    ///slide program.
    ///
    ///For example, in the program
    ///
    ///```text
    ///1 + 2 3 + 4
    ///      ^^^^^- offending tokens
    ///```
    ///
    ///`3 + 4` are not connected to the primary expression statement `1 + 2`, and slide does not
    ///know how this is intended to be evaluated.
    ///
    ///In the future, statement that are separated by a newline will not emit this error. The
    ///following are examples of programs that currently emit this error, but in the future should not:
    ///
    ///```text
    ///a = 1
    ///b = 2 - in the future, parsed as two assignment statements
    ///```
    ///
    ///```text
    ///1 + 2
    ///3 + 4 - in the future, parsed as two expression statements
    ///```
    P0001: ExtraTokens {
        ($span:expr) => {{
            use crate::diagnostics::*;

            Diagnostic::span_err(
                $span,
                "Unexpected extra tokens",
                ExtraTokens::CODE,
                "not connected to a primary statement".to_string(),
            )
            .with_autofix(Autofix::maybe("consider deleting these tokens", Edit::Delete))
        }}
    }

    ///This error fires on token sequences that are expected to parse as an expression, but do not.
    ///
    ///The following are examples of slide programs that emit this error:
    ///
    ///```text
    ///1 + +
    ///    ^- offending token
    ///```
    ///
    ///```text
    ///1 / )
    ///    ^- offending token
    ///```
    ///
    ///```text
    ///1 /
    ///   ^- offending token: end of file
    ///```
    ///
    ///In all cases such programs are malformed and should be refactored to include complete
    ///expressions.
    P0002: ExpectedExpr {
        ($span:expr, $found:expr) => {
            Diagnostic::span_err(
                $span,
                format!("Expected an expression, found {}", $found),
                ExpectedExpr::CODE,
                "expected an expression".to_string(),
            )
        }
    }

    ///All opening delimiters with closing pairs must have that closing delimiter as a
    ///correctly-ordered complement in a slide program. In particular,
    ///
    ///  - `(` and `)` are complements (parantheses)
    ///  - `[` and `]` are complements (brackets)
    ///
    ///The most obvious case for a mismatch is when an incorrect complement is used, for example in
    ///`(1+2]` or `[1+2)`.
    ///
    ///A complement to this is that nesting order must be obeyed. That is, `([1 + 2])` is valid but
    ///`([1 + 2)]` is not.
    ///
    ///Finally, a more subtle case may be when one set of delimiters is not properly closed, as in
    ///the case
    ///
    ///```text
    ///([1 + 2)
    ///       ^- expected closing `]`
    ///```
    P0003: MismatchedClosingDelimiter {
        (expected $expected:expr, at $cur_span:expr, due to $opener:expr, at $open_span:expr; found $found:expr) => {{
            use crate::diagnostics::*;

            Diagnostic::span_err(
                $cur_span,
                format!("Mismatched closing delimiter `{}`", $found),
                MismatchedClosingDelimiter::CODE,
                format!("expected closing `{}`", $expected),
            )
            .with_spanned_note($open_span, format!("opening `{}` here", $opener))
            .with_autofix(Autofix::for_sure("change the delimiter", Edit::Replace($expected.to_string())))
        }}
    }

    ///Patterns are illegal in a "regular" slide program; i.e. a program including a standard
    ///expression.
    ///
    ///In most cases, this error is fired because you intended to run an expression pattern through
    ///slide, or wrote a variable in the form of a pattern.
    ///
    ///Because patterns are abstractions over expressions, they cannot be evaluated in the way an
    ///expression can without first being filled in by an expression. As an analogy, saying you
    ///have "eaten groceries" does not provide concrete information about what you have eaten
    ///without first defining what the groceries are.
    P0004: IllegalPattern {
        ($span:expr, $pat_name:expr) => {{
            use crate::diagnostics::*;

            Diagnostic::span_err(
                $span,
                "Patterns cannot be used in an expression",
                IllegalPattern::CODE,
                "unexpected pattern".to_string(),
            )
            .with_autofix(Autofix::for_sure("use a variable", Edit::Replace($pat_name.substring(1, $pat_name.len() - 1))))
        }}
    }

    ///Variables are illegal in a slide expression pattern.
    ///
    ///In most cases, this error is fired because you intended to evaluate an expression with
    ///slide, or wrote a variable in place of a variable pattern.
    ///
    ///Because expression patterns are meant to abstract over and match expressions, there is
    ///generally not a need to explicitly define the name of a variable to be matched by an
    ///expression pattern. Rather, the concern is generally with the shape of the variable; that
    ///is, that it is actually a variable. For this use case, the "${name}" pattern (where "{name}"
    ///is a text placeholder) serves as a variable-matching pattern.
    ///
    ///As a concrete example, the expression pattern `$a + $b + $a` matches both the expressions
    ///`a + b + a` and `b + a + b`. Both expressions are lowered the same way despite having
    ///different variable names, so variable patterns permit abstraction and common representation
    ///over the names.
    P0005: IllegalVariable {
        ($span:expr, $var_name:expr) => {{
            use crate::diagnostics::*;

            Diagnostic::span_err(
                $span,
                "Variables cannot be used in an expression pattern",
                IllegalVariable::CODE,
                Some("unexpected variable".into()),
            )
            .with_autofix(Autofix::for_sure("use a var pattern", Edit::Replace(format!("${}", $var_name))))
        }}
    }

    ///All closing delimiters with opening pairs must have that opening delimiter as a complement in
    ///a slide program. In particular,
    ///
    ///  - `)` and `(` are complements (parantheses)
    ///  - `]` and `[` are complements (brackets)
    ///
    ///An unmatched closing delimiter error occurs when corresponding opening delimiters are not
    ///present earlier in the slide program. Some examples include:
    ///
    ///```text
    ///1 + 2 )
    ///      ^ unmatched closing delimiter
    ///```
    ///
    ///```text
    ///1 + 2
    ///)]
    ///^ unmatched closing delimiter
    /// ^ unmatched closing delimiter
    ///```
    P0006: UnmatchedClosingDelimiter {
        ($span:expr, $found:expr) => {{
            use crate::diagnostics::*;

            Diagnostic::span_err(
                $span,
                format!(r#"Unmatched closing delimiter "{}""#, $found),
                UnmatchedClosingDelimiter::CODE,
                format!(r#"has no matching opener "{}""#, $found.matcher()),
            )
            .with_autofix(Autofix::maybe("consider deleting this token", Edit::Delete))
        }}
    }
}

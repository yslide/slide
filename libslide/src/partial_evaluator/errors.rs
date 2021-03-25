//! Diagnostic errors produced by the partial evaluator.

use crate::diagnostics::{DiagnosticRecord, DiagnosticRegistry};

macro_rules! define_errors {
    ($($(#[doc = $doc:expr])+ $code:ident: $error:ident $gen_macro:tt)*) => {$(
        $(#[doc = $doc])+
        pub(crate) struct $error;

        impl DiagnosticRecord for $error {
            const CODE: &'static str = stringify!($code);
            const EXPLANATION: &'static str = concat!($($doc, "\n"),+);
        })*

        pub struct PartialEvaluatorErrors;

        impl DiagnosticRegistry for PartialEvaluatorErrors {
            #[allow(clippy::vec_init_then_push)]
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
    ///This error is fired on variable definitions provided to a slide program that can never be
    ///compatible. For example, given the program
    ///
    ///```text
    ///a := 1
    ///a := 12 - 10
    ///```
    ///
    ///"a" is defined as "1" and "2" simultaneously, which are incompatible definitions.
    ///
    ///This error is only fired when slide is able to statically detect incompatibility of
    ///defintions. For example, without having information on what "c" is defined as, the program
    ///
    ///```text
    ///a := c
    ///a := 2c
    ///```
    ///
    ///would not have an incompatible definitions error, because the program is valid when "c = 0".
    ///However, if slide knew about that value of "c", as it would in the program
    ///
    ///```text
    ///a := c
    ///a := 2c
    ///c := 1
    ///```
    ///
    ///an incompatible definitions error could now be fired on the two definitions of "a".
    V0001: IncompatibleDefinitions {
        ($var:expr, $a_def:expr, $b_def:expr) => {
            Diagnostic::span_err(
                $a_def.span,
                format!(r#"Definitions of "{}" are incompatible"#, $var),
                "V0001",
                format!(r#"this definition evaluates to "{}""#, $a_def),
            )
            .with_spanned_err(
                $b_def.span,
                format!(r#"this definition evaluates to "{}""#, $b_def),
            )
            .with_note(format!(
                r#""{}" and "{}" are never equal"#,
                $a_def.rhs, $b_def.rhs
            ))
        }
    }

    // TODO(#263): unify this with other lints.
    ///This warning is fired on variable definitions that may be incompatible. For example, given
    ///the program
    ///
    ///```text
    ///a := b
    ///a := 2*b
    ///```
    ///
    ///The definitions of "a" are maybe-incompatible; in particular, they are compatible iff
    ///"b := 0". This ambiguity is considered error-prone because it does not clearly communicate
    ///intent of the definitions, and there is no information to validate the soundness of a program
    ///in such a state.
    ///
    ///The behavior of maybe-incompatible definitions is considered undefined behavior.
    L0005: MaybeIncompatibleDefinitions {
        ($var:expr, $a_def:expr, $b_def:expr, $dep_vars:expr) => {{
            let last = $dep_vars.len() - 1;
            let mut dep_vars = if last == 0 { "variable " } else { "variables " }.to_string();
            for (i, var) in $dep_vars.into_iter().enumerate() {
                // Cases:
                //   a
                //   a and b
                //   a, b, and c
                dep_vars.push_str(&match i {
                    0 => format!("\"{}\"", var),
                    1 if i == last => format!(" and \"{}\"", var),
                    _ if i == last => format!(", and \"{}\"", var),
                    _ => format!(", \"{}\"", var),
                });
            }
            Diagnostic::span_warn(
                $a_def.span,
                format!(r#"Definitions of "{}" may be incompatible"#, $var),
                "L0002",
                format!(r#"this definition evaluates to "{}""#, $a_def),
            )
            .with_spanned_warn(
                $b_def.span,
                format!(r#"this definition evaluates to "{}""#, $b_def),
            )
            .with_note(format!(
                r#""{}" and "{}" may not be equal"#,
                $a_def.rhs, $b_def.rhs
            ))
            .with_note(
                format!("equivalence of the definitions depends on the {}", dep_vars)
            )
            .with_note(
                "there is not enough information to conclude whether the definitions are compatible"
            )
        }}
    }
}

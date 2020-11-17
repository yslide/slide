//! Diagnostic errors produced by the scanner.

use crate::diagnostics::{DiagnosticRecord, DiagnosticRegistry};

macro_rules! define_errors {
    ($($(#[doc = $doc:expr])+ $code:ident: $error:ident $gen_macro:tt)*) => {$(
        $(#[doc = $doc])+
        pub(crate) struct $error;

        impl DiagnosticRecord for $error {
            const CODE: &'static str = stringify!($code);
            const EXPLANATION: &'static str = concat!($($doc, "\n"),+);
        })*

        /// Diagnostic errors produced by the scanner.
        pub struct ScanErrors;

        impl DiagnosticRegistry for ScanErrors {
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
    ///Tokens in a slide program must be mathematically significant. In particular, slide uses a
    ///subset of canonical mathematical notation to represent expressions.
    ///
    ///This error is fired on a token either because
    ///
    ///  - The token is invalid in the language of common mathematical notation, and/or
    ///  - The token is not yet supported by slide in representing expressions. In this case, we
    ///    would appreciate a bug report at <https://github.com/yslide/slide/issues/new>.
    ///
    ///---
    ///
    ///NB: "canonical mathematical notation" is not well-defined. In general, slide's principle is
    ///to use notation that is intuitive and obvious. Of course, reasonable people can disagree on
    ///what this means.
    S0001: InvalidToken {
        ($span:expr, $did_you_mean:expr) => {{
            let mut diag = Diagnostic::span_err(
                $span,
                "Invalid token",
                InvalidToken::CODE,
                None,
            )
            .with_note("token must be mathematically significant");
            if let Some((did_you_mean, span)) = $did_you_mean {
                diag = diag.with_spanned_help(span, format!(r#"did you mean "{}"?"#, did_you_mean));
            }
            diag
        }}
    }
}

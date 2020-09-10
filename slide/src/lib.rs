//! The slide app. For an overview of slide's design, see [libslide's documentation][libslide].

#![deny(warnings)]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yslide/slide/master/assets/logo.png")]

#[cfg(test)]
mod test;

mod diagnostics;
use diagnostics::emit_slide_diagnostics;

use libslide::diagnostics::Diagnostic;
use libslide::scanner::ScanResult;
use libslide::{
    evaluate, parse_expression, parse_expression_pattern, scan, Emit, EvaluatorContext,
};

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// For wasm, use wee_alloc as a global allocator.
#[cfg(feature = "wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Options to run slide with.
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct Opts {
    /// Slide program.
    pub program: String,
    /// How the result of slide's execution should be emitted.
    pub emit_format: String,
    /// Configuration options for slide emit.
    pub emit_config: Vec<String>,
    /// When true, slide will stop after parsing a program.
    pub parse_only: bool,
    /// When true, slide will expect the program to be an expression pattern.
    pub expr_pat: bool,
    /// When true, slide emit will be colored.
    pub color: bool,
}

/// Output of a slide execution.
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
pub struct SlideResult {
    /// Exit code
    pub code: i32,
    /// Emit for stdout
    pub stdout: String,
    /// Emit for stderr
    pub stderr: String,
}

/// Runs slide end-to-end.
pub fn run_slide(opts: Opts) -> SlideResult {
    let Opts {
        emit_format,
        emit_config,
        program,
        color,
        ..
    } = opts;
    let file = None; // currently programs can only be read from stdin

    let emit_diagnostics = |diagnostics: Vec<Diagnostic>| SlideResult {
        code: 1,
        stdout: String::new(),
        stderr: emit_slide_diagnostics(file, program.clone(), diagnostics, color),
    };
    let emit_tree = move |obj: &dyn Emit| SlideResult {
        code: 0,
        stdout: obj.emit(emit_format.into(), emit_config.into()),
        stderr: String::new(),
    };

    let ScanResult {
        tokens,
        diagnostics,
    } = scan(&*program);
    if !diagnostics.is_empty() {
        return emit_diagnostics(diagnostics);
    }

    if opts.expr_pat {
        let (parse_tree, diagnostics) = parse_expression_pattern(tokens);
        if !diagnostics.is_empty() {
            return emit_diagnostics(diagnostics);
        }
        if opts.parse_only {
            return emit_tree(&parse_tree);
        }
        unreachable!();
    }
    let (parse_tree, diagnostics) = parse_expression(tokens);

    if !diagnostics.is_empty() {
        return emit_diagnostics(diagnostics);
    }

    if opts.parse_only {
        return emit_tree(&parse_tree);
    }

    let simplified = evaluate(parse_tree, &EvaluatorContext::default()).unwrap();
    emit_tree(&simplified)
}

/// Runs slide through a wasm entry point.
/// `opts` must be a JS object with the same fields as [Opts][Opts].
/// Returns a JS object with the same fields as [SlideResult][SlideResult].
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_slide_wasm(opts: JsValue) -> JsValue {
    let opts: Opts = opts.into_serde().unwrap();
    JsValue::from_serde(&run_slide(opts)).unwrap()
}

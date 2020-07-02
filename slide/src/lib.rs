//! The slide app. For an overview of slide's design, see [libslide's documentation][libslide].

#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yslide/slide/master/assets/logo.png")]

#[cfg(test)]
mod test;

mod diagnostics;
use diagnostics::emit_slide_diagnostics;

use libslide::diagnostics::Diagnostic;
use libslide::scanner::ScanResult;
use libslide::{
    evaluate, parse_expression, parse_expression_pattern, scan, EvaluatorContext, Grammar,
};

/// Options to run slide with.
pub struct Opts {
    /// Slide program.
    pub program: String,
    /// How the result of slide's execution should be output.
    pub output_form: OutputForm,
    /// When true, slide will stop after parsing a program.
    pub parse_only: bool,
    /// When true, slide will expect the program to be an expression pattern.
    pub expr_pat: bool,
    /// When true, slide will emit output on any channels or files.
    pub no_emit: bool,
}

/// How the result of slide should be output.
#[derive(Copy, Clone)]
pub enum OutputForm {
    /// Canonical, human-readable form.
    /// For example, `1+1` is output as `1 + 1`.
    Pretty,
    /// S-expression form.
    /// For example, `1+1` is output as `(+ 1 1)`.
    SExpression,
    /// Slide internal debug form.
    /// NB: this form is not stable, and no assumptions should be made about it.
    Debug,
}

/// Runs slide end-to-end.
pub fn run_slide(opts: Opts) -> i32 {
    let output_form = opts.output_form;
    let file = None; // currently programs can only be read from stdin
    let program = opts.program;
    let emit = !opts.no_emit;

    let emit_diagnostics = |diagnostics: Vec<Diagnostic>| {
        if emit {
            emit_slide_diagnostics(file, program.clone(), diagnostics);
        }
        1
    };
    let emit_tree = move |obj: &dyn Grammar| {
        if emit {
            println!("{}", print(obj, output_form));
        }
        0
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

    let simplified = evaluate(parse_tree, EvaluatorContext::default()).unwrap();
    emit_tree(&simplified)
}

fn print(obj: &dyn Grammar, output_form: OutputForm) -> String {
    match output_form {
        OutputForm::Pretty => obj.to_string(),
        OutputForm::SExpression => obj.s_form(),
        OutputForm::Debug => format!("{:#?}", obj),
    }
}

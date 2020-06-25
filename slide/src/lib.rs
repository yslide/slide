#[cfg(test)]
mod test;

mod diagnostics;
use diagnostics::emit_slide_diagnostics;

use libslide::diagnostics::Diagnostic;
use libslide::scanner::ScanResult;
use libslide::{
    evaluate, parse_expression, parse_expression_pattern, scan, EvaluatorContext, Grammar,
};

pub struct Opts {
    pub program: String,
    pub output_form: OutputForm,
    pub parse_only: bool,
    pub expr_pat: bool,
    pub no_emit: bool,
}

#[derive(Copy, Clone)]
pub enum OutputForm {
    Pretty,
    SExpression,
    Debug,
}

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

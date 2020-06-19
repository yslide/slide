mod diagnostics;
use diagnostics::emit_slide_diagnostics;

use libslide::diagnostics::Diagnostic;
use libslide::scanner::ScanResult;
use libslide::{
    evaluate, parse_expression, parse_expression_pattern, scan, EvaluatorContext, Grammar,
};

use std::env;

struct Opts {
    pub program: String,
    pub output_form: OutputForm,
    pub parse_only: bool,
    pub expr_pat: bool,
}

#[derive(Copy, Clone)]
enum OutputForm {
    Pretty,
    SExpression,
    Debug,
}

fn get_opts() -> Opts {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .arg(
            clap::Arg::with_name("program")
                .help("Program to evaluate")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("output-form")
                .short("o")
                .default_value("pretty")
                .takes_value(true)
                .possible_values(&["pretty", "s-expression", "debug"]),
        )
        .arg(
            clap::Arg::with_name("parse-only")
                .long("--parse-only")
                .help("Stop after parsing and dump the AST"),
        )
        .arg(
            clap::Arg::with_name("expr-pat")
                .long("--expr-pat")
                .help("Parse the program as an expression pattern. Implies --parse-only."),
        )
        .get_matches();

    let expr_pat = matches.is_present("expr-pat");
    Opts {
        program: matches.value_of("program").unwrap().into(),
        output_form: match matches.value_of("output-form").unwrap() {
            "pretty" => OutputForm::Pretty,
            "s-expression" => OutputForm::SExpression,
            "debug" => OutputForm::Debug,
            _ => unreachable!(),
        },
        parse_only: matches.is_present("parse-only") || expr_pat,
        expr_pat,
    }
}

fn main_impl() -> Result<(), String> {
    let opts = get_opts();
    let output_form = opts.output_form;
    let file = None; // currently programs can only be read from stdin
    let program = opts.program;

    let emit_diagnostics = |diagnostics: Vec<Diagnostic>| {
        emit_slide_diagnostics(file, program.clone(), diagnostics);
        std::process::exit(1);
    };
    let emit_tree = move |obj: &dyn Grammar| {
        println!("{}", print(obj, output_form));
        std::process::exit(0);
    };

    let ScanResult {
        tokens,
        diagnostics,
    } = scan(&*program);
    if !diagnostics.is_empty() {
        emit_diagnostics(diagnostics);
    }

    if opts.expr_pat {
        let (parse_tree, diagnostics) = parse_expression_pattern(tokens);
        if !diagnostics.is_empty() {
            emit_diagnostics(diagnostics);
        }
        if opts.parse_only {
            emit_tree(&parse_tree);
        }
        unreachable!();
    }
    let (parse_tree, diagnostics) = parse_expression(tokens);

    if !diagnostics.is_empty() {
        emit_diagnostics(diagnostics);
    }

    if opts.parse_only {
        emit_tree(&parse_tree);
    }

    let simplified = evaluate(parse_tree, EvaluatorContext::default()).unwrap();
    emit_tree(&simplified);

    Ok(())
}

fn print(obj: &dyn Grammar, output_form: OutputForm) -> String {
    match output_form {
        OutputForm::Pretty => obj.to_string(),
        OutputForm::SExpression => obj.s_form(),
        OutputForm::Debug => format!("{:#?}", obj),
    }
}

fn main() {
    let out = std::panic::catch_unwind(main_impl);

    if let Err(..) = out {
        eprint!(
            "\nnote: you found an internal slide error (ISE; it's like an ICE, but for slide)!\n"
        );
        eprint!("\nnote: we would appreciate a bug report: https://github.com/ayazhafiz/slide\n");
        std::process::exit(1);
    }
}

mod diagnostics;
use diagnostics::emit_slide_diagnostics;

use libslide::scanner::ScanResult;
use libslide::{evaluate, parse_expression, scan, EvaluatorContext, Grammar};

use std::env;

struct Opts {
    pub program: String,
    pub output_form: OutputForm,
    pub parse_only: bool,
}

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
        .get_matches();

    Opts {
        program: matches.value_of("program").unwrap().into(),
        output_form: match matches.value_of("output-form").unwrap() {
            "pretty" => OutputForm::Pretty,
            "s-expression" => OutputForm::SExpression,
            "debug" => OutputForm::Debug,
            _ => unreachable!(),
        },
        parse_only: matches.is_present("parse-only"),
    }
}

fn main() -> Result<(), String> {
    let opts = get_opts();
    let file = None; // currently programs can only be read from stdin
    let program = opts.program;

    let ScanResult {
        tokens,
        diagnostics,
    } = scan(&*program);
    if !diagnostics.is_empty() {
        emit_slide_diagnostics(file, program, diagnostics);
        std::process::exit(1);
    }

    let (parse_tree, errors) = parse_expression(tokens);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    // TODO: handle errors

    if opts.parse_only {
        println!("{}", print(parse_tree, opts.output_form));
        return Ok(());
    }

    let simplified = evaluate(parse_tree, EvaluatorContext::default()).unwrap();
    println!("{}", print(simplified, opts.output_form));

    Ok(())
}

fn print<T>(obj: T, output_form: OutputForm) -> String
where
    T: Grammar,
{
    match output_form {
        OutputForm::Pretty => obj.to_string(),
        OutputForm::SExpression => obj.s_form(),
        OutputForm::Debug => format!("{:#?}", obj),
    }
}

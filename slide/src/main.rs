use slide::{run_slide, Opts, OutputForm};
use std::env;

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
        no_emit: false,
    }
}

fn main_impl() -> Result<(), String> {
    let opts = get_opts();
    std::process::exit(run_slide(opts))
}

fn main() {
    let out = std::panic::catch_unwind(main_impl);

    if let Err(..) = out {
        eprint!(
            "\nnote: you found an internal slide error (ISE; it's like an ICE, but for slide)!\n"
        );
        eprint!("\nnote: we would appreciate a bug report: https://github.com/yslide/slide\n");
        std::process::exit(2);
    }
}

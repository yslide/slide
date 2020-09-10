use slide::{run_slide, Opts, SlideResult};
use std::env;
use std::io::Write;
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};

fn get_opts(color: bool) -> Opts {
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
                .short("-o")
                .long("--output-form")
                .next_line_help(true)
                .help(
                    "Slide emit format. Possible values:\n\
                    \tpretty:       Human-readable text, like \"1 + 2\".\n\
                    \ts-expression: Prefixed s-expression, like \"(+ 1 2)\".\n\
                    \tlatex:        LaTeX math mode code, like \"$\\left\\(1 + 2\\right\\)$\".\n\
                    \tdebug:        Opaque internal representation. Note: this format is not stable.\n\
                    ",
                )
                .hide_possible_values(true)
                .default_value("pretty")
                .takes_value(true)
                .possible_values(&["pretty", "s-expression", "latex", "debug"]),
        )
        .arg(
            // TODO: validate that -olatex is present.
            clap::Arg::with_name("emit-config")
                .long("--emit-config")
                .next_line_help(true)
                .help(
                    "Emit configuration options. Possible values:\n\
                    \tfrac (latex): Emit divisions as fractions.\n\
                    ",
                )
                .hide_possible_values(true)
                .takes_value(true)
                .possible_values(&["frac"])
                .multiple(true),
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
        // TODO: we should consolidate emit_format and output-form before any stable release.
        emit_format: matches.value_of("output-form").unwrap().into(),
        emit_config: matches
            .values_of("emit-config")
            .map(|opts| opts.map(str::to_owned).collect())
            .unwrap_or_default(),
        parse_only: matches.is_present("parse-only") || expr_pat,
        expr_pat,
        color,
    }
}

fn main_impl() -> Result<(), Box<dyn std::error::Error>> {
    let mut ch_stdout = BufferedStandardStream::stdout(ColorChoice::Auto);
    let mut ch_stderr = BufferedStandardStream::stderr(ColorChoice::Auto);
    let use_color = atty::is(atty::Stream::Stderr) && ch_stderr.supports_color();

    let opts = get_opts(use_color);
    let SlideResult {
        code,
        stdout,
        stderr,
    } = run_slide(opts);

    if !stdout.is_empty() {
        writeln!(&mut ch_stdout, "{}", stdout)?;
        ch_stdout.flush()?;
    }
    if !stderr.is_empty() {
        writeln!(&mut ch_stderr, "{}", stderr)?;
        ch_stderr.flush()?;
    }

    std::process::exit(code)
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

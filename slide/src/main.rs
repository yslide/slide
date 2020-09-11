use slide::{run_slide, Opts, SlideResult};
use std::env;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};

fn get_opts(color: bool) -> Opts {
    let matches = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .author(clap::crate_authors!())
        .arg(
            clap::Arg::with_name("program")
                .help("Program to evaluate")
                .required(true)
                .default_value_if("explain", None, "")
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
            clap::Arg::with_name("lint")
                .long("--lint")
                .help("Emit lint warnings for the program, if any."),
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
        .arg(
            clap::Arg::with_name("explain")
                .long("--explain")
                .value_name("diagnostic")
                .help("Provide a detailed explanation for a diagnostic code.")
                .takes_value(true)
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
        lint: matches.is_present("lint"),
        parse_only: matches.is_present("parse-only") || expr_pat,
        explain_diagnostic: matches.value_of("explain").map(str::to_owned),
        expr_pat,
        color,
    }
}

fn main_impl() -> Result<(), Box<dyn std::error::Error>> {
    let mut ch_stdout = BufferedStandardStream::stdout(ColorChoice::Auto);
    let mut ch_stderr = BufferedStandardStream::stderr(ColorChoice::Auto);
    let is_tty = atty::is(atty::Stream::Stderr);
    let use_color = is_tty && ch_stderr.supports_color();

    let opts = get_opts(use_color);
    let SlideResult {
        code,
        stdout,
        stderr,
        page,
    } = run_slide(opts);

    if !stderr.is_empty() {
        writeln!(&mut ch_stderr, "{}", stderr)?;
        ch_stderr.flush()?;
    }
    if !stdout.is_empty() {
        print_stdout(&stdout, &mut ch_stdout, page)?;
    }

    std::process::exit(code)
}

/// Basically just copied from rust/src/librustc_driver/lib.rs#show_content_with_pager
fn print_stdout(
    stdout: &str,
    mut ch_stdout: &mut BufferedStandardStream,
    page: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut fallback_to_println = false;

    if page {
        let pager_name = env::var_os("PAGER")
            .unwrap_or_else(|| OsString::from(if cfg!(windows) { "more.com" } else { "less" }));

        match Command::new(pager_name).stdin(Stdio::piped()).spawn() {
            Ok(mut pager) => {
                if let Some(pipe) = pager.stdin.as_mut() {
                    if pipe.write_all(stdout.as_bytes()).is_err() {
                        fallback_to_println = true;
                    }
                }

                if pager.wait().is_err() {
                    fallback_to_println = true;
                }
            }
            Err(_) => {
                fallback_to_println = true;
            }
        }
    }

    // If pager fails for whatever reason, we should still print the content to standard output.
    if fallback_to_println || !page {
        writeln!(&mut ch_stdout, "{}", stdout)?;
        ch_stdout.flush()?;
    }

    Ok(())
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

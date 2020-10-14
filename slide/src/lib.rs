//! The slide app. For an overview of slide's design, see [libslide's documentation](libslide).

#![deny(warnings)]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yslide/slide/master/assets/logo.png")]

#[cfg(test)]
mod test;

mod diagnostics;
use diagnostics::{emit_slide_diagnostics, sanitize_source_for_diagnostics};

use libslide::diagnostics::{Diagnostic, DiagnosticKind};
use libslide::scanner::ScanResult;
use libslide::{
    evaluate, lint_expr_pat, lint_stmt, parse_expression_pattern, parse_statement, scan, Emit,
    EmitConfig, EmitFormat, EvaluationResult, ProgramContext, Token,
};

#[cfg(feature = "wasm")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// For wasm, use wee_alloc as a global allocator.
#[cfg(all(feature = "wasm", not(test)))]
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
    /// When true, lint warnings for the program will be emitted, if any.
    pub lint: bool,
    /// When true, slide will stop after parsing a program.
    pub parse_only: bool,
    /// When true, slide will expect the program to be an expression pattern.
    pub expr_pat: bool,
    /// When is [Some](Option::Some) diagnostic code, will explain that code.
    pub explain_diagnostic: Option<String>,
    /// When true, slide emit will be colored.
    pub color: bool,
}

/// Parses [Opts](self::Opts) from the command line or given a parser that acts on the clap
/// [App](clap::App).
pub fn get_opts<P>(parser: P, color: bool) -> Result<Opts, clap::Error>
where
    P: for<'a> FnOnce(clap::App<'a, '_>) -> Result<clap::ArgMatches<'a>, clap::Error>,
{
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
                    \tfrac          (latex):        Emit divisions as fractions.\n\
                    \ttimes         (latex):        Emit \"\\times\" for multiplications.\n\
                    \tdiv           (latex):        Emit \"\\div\" for divisions.\n\
                    \timplicit-mult (pretty|latex): Use implicit multiplication where possible.\n\
                    ",
                )
                .hide_possible_values(true)
                .takes_value(true)
                .possible_values(&["frac", "times", "div", "implicit-mult"])
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
        );
    let matches = parser(matches)?;

    let expr_pat = matches.is_present("expr-pat");
    Ok(Opts {
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
    })
}

/// Output of a slide execution.
#[cfg_attr(feature = "wasm", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct SlideResult {
    /// Exit code
    pub code: i32,
    /// Emit for stdout
    pub stdout: String,
    /// Emit for stderr
    pub stderr: String,
    /// Whether the stdout should be emit as paged
    pub page: bool,
}

/// Builds a [SlideResult](self::SlideResult).
struct SlideResultBuilder<'a> {
    /// File the program is defined in. [None](Option::None) if the program comes from a side
    /// channel like stdin.
    file: Option<&'a str>,
    /// Original slide program source code.
    org_program: &'a str,
    /// Program source code sanitized for diagnostic emission.
    sanitized_program: String,
    emit_format: EmitFormat,
    emit_config: EmitConfig,
    color: bool,
    stdout: String,
    stderr: String,
    page: bool,
}

impl<'a> SlideResultBuilder<'a> {
    fn new(
        file: Option<&'a str>,
        program: &'a str,
        emit_format: impl Into<EmitFormat>,
        emit_config: impl Into<EmitConfig>,
        color: bool,
    ) -> Self {
        Self {
            file,
            org_program: program,
            sanitized_program: sanitize_source_for_diagnostics(program),
            emit_format: emit_format.into(),
            emit_config: emit_config.into(),
            color,
            page: false,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn emit(&mut self, obj: &dyn Emit) {
        self.stdout
            .push_str(&obj.emit(self.emit_format, self.emit_config));
    }

    fn err(&mut self, diagnostics: &[Diagnostic]) {
        self.stderr.push_str(&emit_slide_diagnostics(
            self.file,
            &self.sanitized_program,
            diagnostics,
            self.color,
        ));
    }

    fn page(&mut self, page: bool) {
        self.page = page;
    }

    fn ok(self) -> SlideResult {
        SlideResult {
            code: 0,
            stdout: self.stdout,
            stderr: self.stderr,
            page: self.page,
        }
    }

    fn failed(self) -> SlideResult {
        SlideResult {
            code: 1,
            stdout: self.stdout,
            stderr: self.stderr,
            page: self.page,
        }
    }
}

/// Runs slide end-to-end.
pub fn run_slide(opts: Opts) -> SlideResult {
    let mut result = SlideResultBuilder::new(
        None, // file: currently programs can only be read from stdin
        &opts.program,
        opts.emit_format,
        opts.emit_config,
        opts.color,
    );

    if let Some(diag_code) = opts.explain_diagnostic {
        let codes = Diagnostic::all_codes_with_explanations();
        return match codes.get::<str>(&diag_code) {
            Some(explanation) => {
                result.stdout.push_str(&explanation);
                result.page(true);
                result.ok()
            }
            None => {
                result
                    .stderr
                    .push_str(&format!("{} is not a diagnostic code", diag_code));
                result.failed()
            }
        };
    }

    let ScanResult {
        tokens,
        diagnostics,
    } = scan(&*opts.program);
    result.err(&diagnostics);
    if !diagnostics.is_empty() {
        return result.failed();
    }

    let evaluator = ProgramEvaluator::new(result, tokens, opts.lint, opts.parse_only);

    if opts.expr_pat {
        evaluator.eval_expr_pat()
    } else {
        evaluator.eval_slide_program()
    }
}

/// Evaluates a slide program either as a regular program or an expression pattern.
struct ProgramEvaluator<'a> {
    result: SlideResultBuilder<'a>,
    tokens: Vec<Token>,
    lint: bool,
    parse_only: bool,
}

impl<'a> ProgramEvaluator<'a> {
    fn new(
        result: SlideResultBuilder<'a>,
        tokens: Vec<Token>,
        lint: bool,
        parse_only: bool,
    ) -> Self {
        Self {
            result,
            tokens,
            lint,
            parse_only,
        }
    }

    /// Handles evaluation of a regular slide program (statements, expressions).
    fn eval_slide_program(mut self) -> SlideResult {
        let (parse_tree, diagnostics) = parse_statement(self.tokens, &self.result.org_program);

        self.result.err(&diagnostics);
        if !diagnostics.is_empty() {
            return self.result.failed();
        }

        let program_context = ProgramContext::default().lint(self.lint);
        if self.lint {
            self.result
                .err(&lint_stmt(&parse_tree, self.result.org_program));
        }

        if self.parse_only {
            self.result.emit(&parse_tree);

            self.result.ok()
        } else {
            let EvaluationResult {
                simplified,
                diagnostics,
            } = evaluate(parse_tree, &program_context).unwrap();
            let fatal = diagnostics.iter().any(|d| d.kind == DiagnosticKind::Error);

            self.result.err(&diagnostics);
            if !fatal {
                self.result.emit(&simplified);
            }

            if diagnostics.is_empty() {
                self.result.ok()
            } else {
                self.result.failed()
            }
        }
    }

    /// Handles evaluation of a slide expression pattern.
    fn eval_expr_pat(mut self) -> SlideResult {
        let (parse_tree, diagnostics) = parse_expression_pattern(self.tokens);
        self.result.err(&diagnostics);
        if !diagnostics.is_empty() {
            return self.result.failed();
        }

        if self.lint {
            self.result
                .err(&lint_expr_pat(&parse_tree, self.result.org_program));
        }

        if self.parse_only {
            self.result.emit(&parse_tree);
        } else {
            panic!("Expression patterns can only be parsed.");
        }

        self.result.ok()
    }
}

/// Runs slide through a wasm entry point.
/// `opts` must be a JS object with the same fields as [Opts](self::Opts).
/// Returns a JS object with the same fields as [SlideResult](self::SlideResult).
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run_slide_wasm(opts: JsValue) -> JsValue {
    let opts: Opts = opts.into_serde().unwrap();
    JsValue::from_serde(&run_slide(opts)).unwrap()
}

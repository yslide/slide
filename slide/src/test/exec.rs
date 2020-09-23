use libtest_mimic::Outcome;

pub type SlideOutput = (
    /*stdout*/ String,
    /*stderr*/ String,
    /*exit code*/ String,
);

pub fn run_slide(args: &str, input: &str) -> Result<SlideOutput, Outcome> {
    let sanitized_args = vec!["slide"]
        .into_iter()
        .chain(
            args.lines()
                .filter(|l| !l.is_empty())
                .flat_map(|arg| arg.split(' ')),
        )
        .chain(vec!["--", input].into_iter());

    let opts = match slide::get_opts(|args| args.get_matches_from_safe(sanitized_args), false) {
        Ok(opts) => opts,
        Err(e) => {
            return Ok(if e.use_stderr() {
                (String::new(), e.to_string(), 1.to_string())
            } else {
                (e.to_string(), String::new(), 0.to_string())
            })
        }
    };

    match std::panic::catch_unwind(|| slide::run_slide(opts)) {
        Ok(slide::SlideResult {
            stdout,
            stderr,
            code,
            ..
        }) => Ok((stdout, stderr, code.to_string())),
        Err(_) => Err(print_fail! { Failure: "Test panicked!"; }),
    }
}

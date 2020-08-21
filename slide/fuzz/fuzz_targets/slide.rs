#![no_main]
use libfuzzer_sys::fuzz_target;
use slide::{run_slide, Opts};

fuzz_target!(|program: String| {
    run_slide(Opts {
        program,
        emit_format: "pretty".to_string(),
        parse_only: true,
        expr_pat: false,
        color: false,
    });
});

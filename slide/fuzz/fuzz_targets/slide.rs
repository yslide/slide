#![no_main]
use libfuzzer_sys::fuzz_target;
use libslide::EmitFormat;
use slide::{run_slide, Opts};

fuzz_target!(|program: String| {
    run_slide(Opts {
        program,
        emit_format: EmitFormat::Pretty,
        parse_only: true,
        expr_pat: false,
        no_emit: true,
    });
});

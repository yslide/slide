#![no_main]
use libfuzzer_sys::fuzz_target;
use slide::{run_slide, Opts, OutputForm};

fuzz_target!(|program: String| {
    run_slide(Opts {
        program,
        output_form: OutputForm::Pretty,
        parse_only: true,
        expr_pat: false,
        no_emit: true,
    });
});

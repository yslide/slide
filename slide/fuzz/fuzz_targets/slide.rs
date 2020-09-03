#![no_main]
use libfuzzer_sys::fuzz_target;

use std::process::Command;

fuzz_target!(|program: String| {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("-q");
    cmd.arg("--");
    cmd.arg("--");
    cmd.arg(&program);

    match cmd.output().ok().and_then(|out| out.status.code()) {
        Some(2) => panic!("Failed!"),
        _ => {}
    }
});

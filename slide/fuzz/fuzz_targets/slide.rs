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

    if let Some(2) = cmd.output().ok().and_then(|out| out.status.code()) {
        panic!("Failed!");
    }
});

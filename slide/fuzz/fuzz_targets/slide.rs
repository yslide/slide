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

    if let Some(out) = cmd.output().ok() {
        if out.status.code() == Some(2) {
            panic!(
                "Failed: {}\n{}",
                String::from_utf8(out.stdout).unwrap(),
                String::from_utf8(out.stderr).unwrap()
            );
        }
    }
});

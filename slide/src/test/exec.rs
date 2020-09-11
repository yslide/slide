use super::atomic_lock;
use libtest_mimic::Outcome;
use std::process::Command;

pub type SlideOutput = (
    /*stdout*/ String,
    /*stderr*/ String,
    /*exit code*/ String,
);

pub fn run_slide(args: &str, input: &str) -> Result<SlideOutput, Outcome> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "-q", "--"]);
    cmd.args(
        args.lines()
            .filter(|l| !l.is_empty())
            .flat_map(|arg| arg.split(' ')),
    );
    cmd.args(&["--", &input]);

    let cmd = match cmd.output() {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(print_fail! { Failure: "{}", e; });
        }
    };

    let stdout = String::from_utf8(cmd.stdout).unwrap();
    let stderr = String::from_utf8(cmd.stderr).unwrap();
    let exitcode = match cmd.status.code() {
        Some(n) => n.to_string(),
        None => "no code".to_owned(),
    } + "\n";

    Ok((stdout, stderr, exitcode))
}

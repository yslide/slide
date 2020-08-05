use difference::{Changeset, Difference};
use libtest_mimic::{run_tests, Arguments, LineFormat, LinePrinter, Outcome, Test};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Collects all `.slide` system test files, starting from slide/src/test and visiting all nested
/// directories.
fn collect_tests() -> Result<Vec<Test<PathBuf>>, Box<dyn Error>> {
    let root_test_path = Path::new("src/test");
    let mut dirs_to_visit = vec![root_test_path.to_path_buf()];
    let mut tests = Vec::with_capacity(200);
    while let Some(dir) = dirs_to_visit.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let entry_type = entry.file_type()?;
            if entry_type.is_dir() {
                dirs_to_visit.push(path);
                continue;
            }
            if path.extension() == Some(OsStr::new("slide")) {
                let name = path.strip_prefix(root_test_path)?.display().to_string();

                tests.push(Test {
                    name,
                    kind: "system".into(),
                    is_ignored: false,
                    is_bench: false,
                    data: path,
                })
            }
        }
    }
    Ok(tests)
}

/// Describes a test case
struct TestCase {
    /// Annotation name -> Annotation message
    annotations: HashMap<String, String>,
    annotation_order: Vec<String>,
    args: String,
    input: String,
    stdout: String,
    stderr: String,
    exitcode: String,
}

/// Returns the delimiter for a test case clause in a .slide test file.
fn get_clause_delim(clause: &str) -> String {
    let prefix = match clause {
        "args" => "!!!",
        "in" => "===",
        "exitcode" | "stdout" | "stderr" => "~~~",
        _ => unreachable!(),
    };
    format!("{}{}", prefix, clause)
}

/// Returns whether a clause can be auto-generated with --bless.
fn can_be_blessed(clause: &str) -> bool {
    matches!(clause, "exitcode" | "stdout" | "stderr")
}

/// Returns the command to bless a test file.
fn get_bless_cmd(test_name: &str) -> String {
    format!("ladder test {} --bless", test_name)
}

/// Creates a TestCase from a .slide test file.
fn mk_test_case(test_name: &str, mut content: String, bless: bool) -> Result<TestCase, Outcome> {
    // First we need to parse out annotations at the top of the file.
    let mut annotations = HashMap::<String, String>::new();
    let mut annotation_order = Vec::new();
    while content.starts_with('@') {
        let line_split = content.find('\n').unwrap();
        let rest_content = content.split_off(line_split);
        let annotation = content;
        content = rest_content;
        let mut annotation_parts = annotation.splitn(1, ':');
        let annotation = annotation_parts.next().unwrap();
        let annotation_msg = annotation_parts.next().map(|s| s.trim()).unwrap_or("");
        annotations.insert(annotation.into(), annotation_msg.into());
        annotation_order.push(annotation.into());
    }

    // Next we get all the clauses.
    let clause_names = ["args", "in", "stdout", "stderr", "exitcode"];
    let mut clauses = Vec::with_capacity(clause_names.len());
    for clause in clause_names.iter() {
        let is_blessable_clause = can_be_blessed(clause);
        if bless && is_blessable_clause {
            // These will get updated later, so just make them empty for now.
            clauses.push("".into());
            continue;
        }

        let clause_delim = get_clause_delim(clause);
        let mut splits: Vec<_> = content
            .split(&format!("{}\n", clause_delim))
            .map(String::from)
            .collect();
        if splits.len() != 3 {
            if clause == &"args" {
                // Args are optional.
                clauses.push("".into());
                content = splits.pop().unwrap();
                continue;
            }
            let test_name = test_name.to_owned();
            return Err(Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(
                        &format!("{} clause missing in test case.", clause_delim),
                        &LineFormat::Failure,
                    );
                    printer.print_line(
                        format!(
                            r#"
Hint: Add a

      {clause_delim}
      <text>
      {clause_delim}

      section to the test file.
"#,
                            clause_delim = clause_delim
                        )
                        .trim(),
                        &LineFormat::Suggestion,
                    );
                    if is_blessable_clause {
                        printer.print_line(
                            &format!(
                                "Hint: You can run `{}` to do this for you.",
                                get_bless_cmd(&test_name)
                            ),
                            &LineFormat::Suggestion,
                        );
                    }
                })),
            });
        }
        content = splits.pop().unwrap(); // next content is the last split
        let mut clause_content = splits.pop().unwrap(); // clause content is the second split
        if clause == &"in" {
            // The split input always has a trailing newline that isn't intended for the test.
            // ===in
            // <program>
            // ===in
            //          ^ newline here
            clause_content.pop();
        }
        clauses.push(clause_content);
    }
    let mut clauses = clauses.into_iter();

    Ok(TestCase {
        annotations,
        annotation_order,
        args: clauses.next().unwrap(),
        input: clauses.next().unwrap(),
        stdout: clauses.next().unwrap(),
        stderr: clauses.next().unwrap(),
        exitcode: clauses.next().unwrap(),
    })
}

/// Prints a diff between two texts.
fn print_diff(printer: &mut dyn LinePrinter, text1: &str, text2: &str) {
    let Changeset { diffs, .. } = Changeset::new(text1, text2, "\n");

    for diff in diffs {
        let (content, prefix, fmt) = match diff {
            Difference::Same(ref x) => (x, " ", &LineFormat::Text),
            Difference::Add(ref x) => (x, "+", &LineFormat::Success),
            Difference::Rem(ref x) => (x, "-", &LineFormat::Failure),
        };
        printer.print_line(&format!("{}{}", prefix, content), fmt);
    }
}

/// Generates the contents of a blessed file for a test case.
fn mk_bless_file(test_case: &TestCase, stdout: &str, stderr: &str, exitcode: &str) -> String {
    let mut content = String::with_capacity(256);

    if !test_case.annotations.is_empty() {
        for annotation in test_case.annotation_order.iter() {
            let msg = test_case.annotations.get(annotation).unwrap();
            if !msg.is_empty() {
                content.push_str(&format!("{}: {}\n", annotation, msg));
            } else {
                content.push_str(&annotation);
                content.push('\n');
            }
        }
        content.push('\n');
    }

    let mut push = |clause: &str, clause_content: &str| {
        let clause_delim = get_clause_delim(clause);
        content.push_str(&format!("{}\n", clause_delim));
        content.push_str(clause_content);
        if clause == "in" {
            content.push('\n');
        }
        content.push_str(&format!("{}\n\n", clause_delim));
    };
    if !test_case.args.is_empty() {
        push("args", &test_case.args);
    }
    push("in", &test_case.input);
    push("stdout", stdout);
    push("stderr", stderr);
    push("exitcode", exitcode);
    content.pop(); // drop trailing newline
    content
}

/// Wraps an object in a thread-safe atomic mutex.
fn atomic_lock<T>(obj: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(obj))
}

/// Runs a slide system test.
fn drive_test(test: &Test<PathBuf>) -> Outcome {
    let bless = env::var("BLESS") == Ok("1".into());
    let fail_todo = env::var("FAIL_TODO") == Ok("1".into());

    let path = &test.data;
    let test_name = test.name.to_string();
    let content = fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
    let content = String::from_utf8(content)
        .unwrap_or_else(|_| panic!("{} is not valid UTF-8", path.display()));

    let test_case = match mk_test_case(&test_name, content, bless) {
        Ok(tc) => tc,
        Err(outcome) => return outcome,
    };

    if fail_todo && test_case.annotations.contains_key("@TODO") {
        return Outcome::Failed {
            msg: Some(atomic_lock(|printer: &mut dyn LinePrinter| {
                printer.print_line(
                    "Test is marked @TODO, which the test runner is set to fail on.",
                    &LineFormat::Failure,
                );
            })),
        };
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("-q");
    cmd.arg("--");
    for arg in test_case.args.lines() {
        for sub_arg in arg.split(' ') {
            if arg.is_empty() {
                continue;
            }
            cmd.arg(sub_arg);
        }
    }
    cmd.arg("--");
    cmd.arg(&test_case.input);

    let cmd = match cmd.output() {
        Ok(cmd) => cmd,
        Err(e) => {
            return Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(&e.to_string(), &LineFormat::Failure);
                })),
            };
        }
    };

    let stdout = String::from_utf8(cmd.stdout).unwrap();
    let stderr = String::from_utf8(cmd.stderr).unwrap();
    let exitcode = cmd.status.code().unwrap().to_string() + "\n";

    if bless {
        let blessed = mk_bless_file(&test_case, &stdout, &stderr, &exitcode);
        return match fs::write(path, blessed) {
            Ok(_) => Outcome::Passed,
            Err(e) => Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(&e.to_string(), &LineFormat::Failure);
                })),
            },
        };
    }

    if stdout != test_case.stdout || stderr != test_case.stderr || exitcode != test_case.exitcode {
        return Outcome::Failed {
            msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                if stdout != test_case.stdout {
                    printer.print_line("Mismatch in stdout:", &LineFormat::Text);
                    print_diff(printer, &test_case.stdout, &stdout);
                }
                if stderr != test_case.stderr {
                    printer.print_line("Mismatch in stderr:", &LineFormat::Text);
                    print_diff(printer, &test_case.stderr, &stderr);
                }
                if exitcode != test_case.exitcode {
                    printer.print_line("Mismatch in exit code:", &LineFormat::Text);
                    print_diff(printer, &test_case.exitcode, &exitcode);
                }
                printer.print_line(
                    &format!(
                        "Help: If this is expected, try running `{}`.",
                        get_bless_cmd(&test_name),
                    ),
                    &LineFormat::Suggestion,
                );
            })),
        };
    }

    Outcome::Passed
}

#[allow(unused)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    let tests = collect_tests()?;
    run_tests(&args, tests, drive_test).exit();
}

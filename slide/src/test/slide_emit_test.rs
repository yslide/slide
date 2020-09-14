use super::*;

use libtest_mimic::{LinePrinter, Outcome, Test};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Describes a test case for testing the slide emit of a program.
#[derive(Clone)]
pub struct SlideEmitTest {
    /// Annotation name -> Annotation message
    pub annotations: HashMap<String, String>,
    pub annotation_order: Vec<String>,
    pub args: String,
    pub input: String,
    pub stdout: String,
    pub stderr: String,
    pub exitcode: String,
}

impl SlideEmitTest {
    /// Creates a slide emit test case from a .slide test file.
    pub fn new(test_file: &Test<PathBuf>, mut content: String) -> Result<Self, ErrorMsgRef> {
        // First we need to parse out annotations at the top of the file.
        let mut annotations = HashMap::<String, String>::new();
        let mut annotation_order = Vec::new();
        while content.starts_with('@') {
            let line_split = content.find('\n').unwrap();
            let rest_content = content.split_off(line_split);
            let annotation = content;
            content = rest_content;

            let mut annotation_parts = annotation.splitn(2, ':');
            let annotation = annotation_parts.next().unwrap();
            let annotation_msg = annotation_parts.next().map(|s| s.trim()).unwrap_or("");

            annotations.insert(annotation.into(), annotation_msg.into());
            annotation_order.push(annotation.into());
        }

        // Next we get all the clauses.
        let clause_names = ["args", "in", "stdout", "stderr", "exitcode"];
        let mut clauses = Vec::with_capacity(clause_names.len());
        for clause in clause_names.iter() {
            let mut splits: Vec<_> = content
                .split(&format!("{}\n", get_clause_delim(clause)))
                .map(String::from)
                .collect();

            if splits.len() != 3 {
                if clause == &"args" || (*BLESS && can_be_blessed(clause)) {
                    // Args are optional, so we can skip them if not found.
                    // If running in bless mode, blessable clauses will get updated later, so just
                    // make them empty for now.
                    clauses.push("".into());
                    content = splits.pop().unwrap();
                    continue;
                }

                let err = Self::missing_clause_failure(test_file, clause.to_string());
                return Err(err);
            }

            content = splits.pop().unwrap(); // next content is the last split
            let mut clause_content = splits.pop().unwrap(); // clause content is the second split

            if clause == &"in" {
                // The split input always has a trailing newline that isn't intended for the test.
                // ===in
                // <program>
                //          ^ newline here
                // ===in
                clause_content.pop();
            }
            clauses.push(clause_content);
        }
        let mut clauses = clauses.into_iter();

        Ok(SlideEmitTest {
            annotations,
            annotation_order,
            args: clauses.next().unwrap(),
            input: clauses.next().unwrap(),
            stdout: clauses.next().unwrap(),
            stderr: clauses.next().unwrap(),
            exitcode: clauses.next().unwrap(),
        })
    }

    /// Executes a slide emit test, ensuring the stdout and stderr of the slide program is as
    /// expected from the test file.
    /// If run in bless mode, the test file is updated with the actual stdout and stderr.
    /// If run in fail-todo mode, the test fails on any @TODO annotations.
    pub fn drive_test(self, test_name: String, test_path: PathBuf) -> Outcome {
        if *FAIL_TODO && self.annotations.contains_key("@TODO") {
            return print_fail! {
                Failure: "Test is marked @TODO, which the test runner is set to fail on.";
            };
        }

        let (stdout, stderr, exitcode) = match run_slide(&self.args, &self.input) {
            Ok(res) => res,
            Err(outcome) => return outcome,
        };

        if *BLESS {
            let blessed = self.make_bless_file(&stdout, &stderr, &exitcode);
            return match fs::write(test_path, blessed) {
                Ok(_) => Outcome::Passed,
                Err(e) => print_fail! { Failure: "{}", e; },
            };
        }

        // Right ends of bless content may be inaccurate because we always force a newline, so
        // just check that the actual content is correct.

        macro_rules! t {
            ($expr:expr) => {
                $expr.trim_end()
            };
        }

        if t!(stdout) != t!(self.stdout)
            || t!(stderr) != t!(self.stderr)
            || t!(exitcode) != t!(self.exitcode)
        {
            return fail! { move |printer: &mut dyn LinePrinter| {
                if t!(stdout) != t!(self.stdout) {
                    printer! { printer Text: "Mismatch in stdout:"; };
                    print_diff(printer, &t!(self.stdout), &t!(stdout));
                }
                if t!(stderr) != t!(self.stderr) {
                    printer! { printer Text: "Mismatch in stderr:"; };
                    print_diff(printer, &t!(self.stderr), &t!(stderr));
                }
                if t!(exitcode) != t!(self.exitcode) {
                    printer! { printer Text: "Mismatch in exit code:"; };
                    print_diff(printer, &t!(self.exitcode), &t!(exitcode));
                }
                printer! { printer
                    Suggestion: "Help: If this is expected, try running `{}`.",
                                get_bless_cmd(&test_name);
                };
            }};
        }

        Outcome::Passed
    }

    /// Generates the actual ("bless"ed) contents of a test case.
    fn make_bless_file(&self, stdout: &str, stderr: &str, exitcode: &str) -> String {
        let mut content = String::with_capacity(256);

        if !self.annotations.is_empty() {
            for annotation in self.annotation_order.iter() {
                let msg = self.annotations.get(annotation).unwrap();
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
            if !clause_content.is_empty() && !clause_content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("{}\n\n", clause_delim));
        };

        if !self.args.is_empty() {
            push("args", &self.args);
        }
        push("in", &self.input);
        push("stdout", stdout);
        push("stderr", stderr);
        push("exitcode", exitcode);
        content.pop(); // drop trailing newline
        content
    }

    /// Creates an error for a missing clause in a test file and returns a reference to where it is
    /// stored in TEST_CONSTRUCTION_FAIL.
    fn missing_clause_failure(test_file: &Test<PathBuf>, clause: String) -> ErrorMsgRef {
        let clause_delim = get_clause_delim(&clause);
        let test_name = test_file.name.to_owned();
        let printer = move |printer: &mut dyn LinePrinter| {
            printer! { printer
                Failure:    "{} clause missing in test case.", clause_delim;
                Suggestion: "Add a\n\
                            \n\
                            \t{}\n\
                            \t<text>\n\
                            \t{}\n\
                            \n\
                            section to the test file.",
                            clause_delim,
                            clause_delim;
            };
            if can_be_blessed(&clause) {
                printer! { printer
                    Suggestion: "You can run `{}` to do this for you.", get_bless_cmd(&test_name);
                };
            }
        };
        let mut outcome_lk = TEST_CONSTRUCTION_FAIL
            .write()
            .expect("Test outcome map poisoned.");
        let i = outcome_lk.len();
        outcome_lk.insert(i, Box::new(printer));
        i
    }
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

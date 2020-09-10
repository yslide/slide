use difference::{Changeset, Difference};
use lazy_static::lazy_static;
use libtest_mimic::{run_tests, Arguments, LineFormat, LinePrinter, Outcome, Test};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};

type Printer = dyn Fn(&mut dyn LinePrinter) + Send + Sync;

lazy_static! {
    static ref TEST_CONSTRUCTION_FAIL: RwLock<HashMap<usize, Box<Printer>>> =
        RwLock::new(HashMap::new());
    static ref BLESS: bool = env::var("BLESS") == Ok("1".into());
    static ref FAIL_TODO: bool = env::var("FAIL_TODO") == Ok("1".into());
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

/// Wraps an object in a thread-safe atomic mutex.
fn atomic_lock<T>(obj: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(obj))
}

/// Describes a test case for testing the slide emit of a program.
#[derive(Clone)]
struct SlideEmitTest {
    /// Annotation name -> Annotation message
    annotations: HashMap<String, String>,
    annotation_order: Vec<String>,
    args: String,
    input: String,
    stdout: String,
    stderr: String,
    exitcode: String,
}

#[derive(Clone)]
struct LaTeXEmitTest {
    /// Annotation name -> Annotation message
    args: String,
    input: String,
}

impl From<&SlideEmitTest> for LaTeXEmitTest {
    fn from(test: &SlideEmitTest) -> Self {
        Self {
            args: test.args.clone(),
            input: test.input.clone(),
        }
    }
}

#[derive(Clone)]
enum TestCaseVariant {
    SlideEmit(SlideEmitTest),
    LaTeXEmit(LaTeXEmitTest),
    FailedTestConstruction(usize),
}

#[derive(Clone)]
struct TestCase {
    path: PathBuf,
    variant: TestCaseVariant,
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

/// Generates the contents of a blessed file for a test case.
fn mk_bless_file(test_case: &SlideEmitTest, stdout: &str, stderr: &str, exitcode: &str) -> String {
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
        if !clause_content.is_empty() && !clause_content.ends_with('\n') {
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

/// Creates a slide-run TestCase from a .slide test file.
fn mk_slide_emit_test_case(
    test_file: &Test<PathBuf>,
    mut content: String,
) -> Result<SlideEmitTest, usize> {
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

        let clause_delim = get_clause_delim(clause);
        let mut splits: Vec<_> = content
            .split(&format!("{}\n", clause_delim))
            .map(String::from)
            .collect();
        if splits.len() != 3 {
            if clause == &"args" || (*BLESS && is_blessable_clause) {
                // Args are optional.
                // If running in bless mode, blessable clauses will get updated later, so just make
                // them empty for now.
                clauses.push("".into());
                content = splits.pop().unwrap();
                continue;
            }
            let test_name = test_file.name.to_owned();
            let printer = move |printer: &mut dyn LinePrinter| {
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
            };
            let mut outcome_lk = TEST_CONSTRUCTION_FAIL
                .write()
                .expect("Test outcome map poisoned.");
            let i = outcome_lk.len();
            outcome_lk.insert(i, Box::new(printer));
            return Err(i);
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

struct TestCaseBuilder<'a> {
    collector: &'a mut Vec<Test<TestCase>>,
    test_file: &'a Test<PathBuf>,
}

impl<'a> TestCaseBuilder<'a> {
    fn new(collector: &'a mut Vec<Test<TestCase>>, test_file: &'a Test<PathBuf>) -> Self {
        Self {
            collector,
            test_file,
        }
    }

    fn add(&mut self, test_case: TestCaseVariant) {
        self.add_suffixed(test_case, "");
    }

    fn add_suffixed(&mut self, test_case: TestCaseVariant, suffix: &str) {
        self.collector.push(Test {
            name: format!("{}{}", self.test_file.name, suffix),
            data: TestCase {
                path: self.test_file.data.clone(),
                variant: test_case,
            },
            kind: "system".to_owned(),
            is_ignored: false,
            is_bench: false,
        });
    }
}

fn mk_test_cases(test_files: Vec<Test<PathBuf>>) -> Vec<Test<TestCase>> {
    let mut cases = Vec::with_capacity(test_files.len() * 2);
    for test_file in test_files {
        let path = &test_file.data;
        let content =
            fs::read(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
        let content = String::from_utf8(content)
            .unwrap_or_else(|_| panic!("{} is not valid UTF-8", path.display()));

        let mut builder = TestCaseBuilder::new(&mut cases, &test_file);

        let slide_emit_test = match mk_slide_emit_test_case(&test_file, content) {
            Err(failed_test) => {
                builder.add(TestCaseVariant::FailedTestConstruction(failed_test));
                continue;
            }
            Ok(test) => test,
        };

        if slide_emit_test.args.contains(" latex") && slide_emit_test.exitcode.trim() == "0" {
            // The check could be better, but this will do for now.
            builder.add_suffixed(
                TestCaseVariant::LaTeXEmit(LaTeXEmitTest::from(&slide_emit_test)),
                " latex emit",
            );
        }

        builder.add(TestCaseVariant::SlideEmit(slide_emit_test));
    }
    cases
}

/// Collects all `.slide` system test files, starting from slide/src/test and visiting all nested
/// directories.
fn collect_test_files() -> Result<Vec<Test<PathBuf>>, Box<dyn Error>> {
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
                    kind: "system".to_owned(),
                    is_ignored: false,
                    is_bench: false,
                    data: path,
                });
            }
        }
    }
    Ok(tests)
}

type SlideOutput = (
    /*stdout*/ String,
    /*stderr*/ String,
    /*exit code*/ String,
);

fn run_slide(args: &str, input: &str) -> Result<SlideOutput, Outcome> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("-q");
    cmd.arg("--");
    for arg in args.lines() {
        for sub_arg in arg.split(' ') {
            if arg.is_empty() {
                continue;
            }
            cmd.arg(sub_arg);
        }
    }
    cmd.arg("--");
    cmd.arg(&input);

    let cmd = match cmd.output() {
        Ok(cmd) => cmd,
        Err(e) => {
            return Err(Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(&e.to_string(), &LineFormat::Failure);
                })),
            });
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

fn handle_slide_emit_test(
    test_name: String,
    test_path: PathBuf,
    test_case: SlideEmitTest,
) -> Outcome {
    if *FAIL_TODO && test_case.annotations.contains_key("@TODO") {
        return Outcome::Failed {
            msg: Some(atomic_lock(|printer: &mut dyn LinePrinter| {
                printer.print_line(
                    "Test is marked @TODO, which the test runner is set to fail on.",
                    &LineFormat::Failure,
                );
            })),
        };
    }

    let (stdout, stderr, exitcode) = match run_slide(&test_case.args, &test_case.input) {
        Ok(res) => res,
        Err(outcome) => return outcome,
    };

    if *BLESS {
        let blessed = mk_bless_file(&test_case, &stdout, &stderr, &exitcode);
        return match fs::write(test_path, blessed) {
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

fn handle_latex_emit_test(
    test_name: String,
    test_path: PathBuf,
    test_case: LaTeXEmitTest,
) -> Outcome {
    let (stdout, _stderr, _exitcode) = match run_slide(&test_case.args, &test_case.input) {
        Ok(res) => res,
        Err(outcome) => return outcome,
    };
    let stdout = stdout.trim();

    let math_mode_inner = &stdout[1..(stdout.len() - 1)];
    let latex_img_url = format!(
        "https://latex.codecogs.com/png.latex?\\dpi{{400}}{}",
        math_mode_inner
    );

    // TODO: pool connections
    let mb_img_bytes = reqwest::blocking::get(
        reqwest::Url::parse(&latex_img_url)
            .unwrap_or_else(|_| panic!("LaTeX image url is invalid: {}", latex_img_url)),
    )
    .and_then(|resp| resp.bytes());
    let img_bytes = match mb_img_bytes {
        Ok(bytes) => bytes,
        Err(err) => {
            return Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(&err.to_string(), &LineFormat::Failure);
                })),
            };
        }
    };

    let im_actual =
        image::load_from_memory_with_format(&img_bytes, image::ImageFormat::PNG).unwrap();

    let mut actual_img_path = test_path.clone();
    actual_img_path.set_extension("latex.actual.png");

    let mut golden_img_path = test_path.clone();
    golden_img_path.set_extension("latex.png");

    if *BLESS {
        im_actual.save(golden_img_path).unwrap();
        return Outcome::Passed;
    }

    let mut im_golden = match image::open(&golden_img_path) {
        Ok(im) => im,
        Err(_) => {
            return Outcome::Failed {
                msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                    printer.print_line(
                        &format!(
                            "Failed to load golden LaTeX emit image {}",
                            &golden_img_path.display().to_string(),
                        ),
                        &LineFormat::Failure,
                    );
                    printer.print_line("Does it exist?", &LineFormat::Suggestion);
                    printer.print_line(
                        &format!(
                            "Hint: You can run `{}` to do add the golden for you.",
                            get_bless_cmd(&test_name)
                        ),
                        &LineFormat::Suggestion,
                    );
                })),
            };
        }
    };

    if im_golden.raw_pixels() != im_actual.raw_pixels() {
        let mut im_actual_for_diff = im_actual.clone();
        let im_diff =
            lcs_image_diff::compare(&mut im_golden, &mut im_actual_for_diff, 100.0 / 256.0)
                .unwrap();
        let mut diff_img_path = test_path;
        diff_img_path.set_extension("latex.diff.png");

        im_actual.save(&actual_img_path).unwrap();
        im_diff.save(&diff_img_path).unwrap();

        return Outcome::Failed {
            msg: Some(atomic_lock(move |printer: &mut dyn LinePrinter| {
                printer.print_line(
                    "Golden and actual LaTeX emit images differ.",
                    &LineFormat::Failure,
                );
                printer.print_line(
                    &format!(
                        "Note: actual image at {}",
                        actual_img_path.display().to_string()
                    ),
                    &LineFormat::Text,
                );
                printer.print_line(
                    &format!(
                        "Note: diff image between golden and actual at {}",
                        diff_img_path.display().to_string()
                    ),
                    &LineFormat::Text,
                );
                printer.print_line(
                    &format!(
                        "Hint: If this is expected, run `{}` to update the golden.",
                        get_bless_cmd(&test_name)
                    ),
                    &LineFormat::Suggestion,
                );
            })),
        };
    }

    Outcome::Passed
}

/// Runs a slide system test.
fn drive_test(test: &Test<TestCase>) -> Outcome {
    let test_name = test.name.clone();
    let test_path = test.data.path.clone();
    match test.data.variant.clone() {
        TestCaseVariant::FailedTestConstruction(outcome) => {
            let read_lk = TEST_CONSTRUCTION_FAIL.read().unwrap();
            let closure: &Printer = read_lk.get(&outcome).unwrap().as_ref();
            unsafe {
                Outcome::Failed {
                    msg: Some(atomic_lock(&*(closure as *const Printer))),
                }
            }
        }
        TestCaseVariant::SlideEmit(tc) => handle_slide_emit_test(test_name, test_path, tc),
        TestCaseVariant::LaTeXEmit(tc) => handle_latex_emit_test(test_name, test_path, tc),
    }
}

#[allow(unused)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    let test_files = collect_test_files()?;
    let tests = mk_test_cases(test_files);
    run_tests(&args, tests, drive_test).exit();
}

use difference::{Changeset, Difference};
use libtest_mimic::{run_tests, Arguments, LineFormat, Outcome, Test};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[macro_use]
mod common;
mod exec;
mod latex_emit_test;
mod slide_emit_test;

use common::*;
use exec::*;
use latex_emit_test::LaTeXEmitTest;
use slide_emit_test::SlideEmitTest;

#[allow(unused)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    let test_files = collect_test_files()?;
    let tests = test_files.into_iter().flat_map(TestCase::new).collect();
    run_tests(&args, tests, TestCase::drive_test).exit();
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

#[derive(Clone)]
struct TestCase {
    path: PathBuf,
    variant: TestCaseVariant,
}

impl TestCase {
    /// Executes a slide system test.
    fn drive_test(test: &Test<Self>) -> Outcome {
        let test_name = test.name.clone();
        let test_path = test.data.path.clone();
        match test.data.variant.clone() {
            TestCaseVariant::FailedTestConstruction(outcome) => {
                let read_lk = TEST_CONSTRUCTION_FAIL.read().unwrap();
                let closure: &Printer = read_lk.get(&outcome).unwrap().as_ref();
                unsafe { fail!(&*(closure as *const Printer)) }
            }
            TestCaseVariant::SlideEmit(tc) => tc.drive_test(test_name, test_path),
            TestCaseVariant::LaTeXEmit(tc) => tc.drive_test(test_name, test_path),
        }
    }

    /// Creates a set of test cases from a slide system test file.
    fn new(test_file: Test<PathBuf>) -> Vec<Test<Self>> {
        let mut cases = Vec::with_capacity(2);
        let path = &test_file.data;
        let content = fs::read(path).map(String::from_utf8).unwrap().unwrap();

        let mut builder = TestCaseBuilder::new(&mut cases, &test_file);

        let slide_emit_test = match SlideEmitTest::new(&test_file, content) {
            Ok(test) => test,
            Err(failed_test) => {
                builder.add(failed_test);
                return cases;
            }
        };

        if (slide_emit_test.args.contains(" latex") || slide_emit_test.args.contains("=latex"))
            && slide_emit_test.exitcode.trim() == "0"
        {
            // The check could be better, but this will do for now.
            builder.add_suffixed(LaTeXEmitTest::from(&slide_emit_test), " latex emit");
        }

        builder.add(slide_emit_test);
        cases
    }
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

    fn add(&mut self, test_case: impl Into<TestCaseVariant>) {
        self.add_suffixed(test_case, "");
    }

    fn add_suffixed(&mut self, test_case: impl Into<TestCaseVariant>, suffix: &str) {
        self.collector.push(Test {
            name: format!("{}{}", self.test_file.name, suffix),
            data: TestCase {
                path: self.test_file.data.clone(),
                variant: test_case.into(),
            },
            kind: "system".to_owned(),
            is_ignored: false,
            is_bench: false,
        });
    }
}

#[derive(Clone)]
enum TestCaseVariant {
    SlideEmit(SlideEmitTest),
    LaTeXEmit(LaTeXEmitTest),
    FailedTestConstruction(ErrorMsgRef),
}

macro_rules! variant_from_test {
    ($($variant:ident from $test:ident)*) => {$(
        impl From<$test> for TestCaseVariant {
            fn from(test: $test) -> Self {
                Self::$variant(test)
            }
        }
    )*};
}

variant_from_test! {
    SlideEmit from SlideEmitTest
    LaTeXEmit from LaTeXEmitTest
    FailedTestConstruction from ErrorMsgRef
}

/// Wraps an object in a thread-safe atomic mutex.
#[inline]
fn atomic_lock<T>(obj: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(obj))
}

/// Returns the command to bless a test file.
fn get_bless_cmd(test_name: &str) -> String {
    format!("ladder test \"{}\" --bless", test_name)
}

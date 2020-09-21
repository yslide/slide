use super::*;

#[derive(Clone)]
pub struct LaTeXEmitTest {
    /// Annotation name -> Annotation message
    args: String,
    input: String,
}

impl LaTeXEmitTest {
    /// Executes a LaTeX emit test, ensuring the code of a slide program's LaTeX emit renders as
    /// expected from a golden image file.
    /// If run in bless mode, the golden image is updated to be the actual image.
    pub fn drive_test(self, test_name: String, test_path: PathBuf) -> Outcome {
        let (stdout, _stderr, _exitcode) = match run_slide(&self.args, &self.input) {
            Ok(res) => res,
            Err(outcome) => return outcome,
        };
        let stdout = stdout.trim();

        let math_mode_inner = if stdout.starts_with('$') {
            &stdout[1..(stdout.len() - 1)]
        } else {
            &stdout
        };
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
                return print_fail! { Failure: "{}", err; };
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
                return print_fail! {
                    Failure:    "Failed to load golden LaTeX emit image {}",
                                real_path!(golden_img_path.display());
                    Suggestion: "Does it exist?";
                    Suggestion: "You can run `{}` to create the golden for you.",
                                get_bless_cmd(&test_name);
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

            return print_fail! {
                Failure:    "Golden and actual LaTeX emit images differ.";
                Suggestion: "Actual image at\n\t{}", real_path!(actual_img_path.display());
                Suggestion: "Diff between golden and actual at\n\t{}", real_path!(diff_img_path.display());
                Suggestion: "If this is expected, run `{}` to update the golden.",
                            get_bless_cmd(&test_name);
            };
        }

        Outcome::Passed
    }
}

impl From<&SlideEmitTest> for LaTeXEmitTest {
    fn from(test: &SlideEmitTest) -> Self {
        Self {
            args: test.args.clone(),
            input: test.input.clone(),
        }
    }
}

use slide::{get_opts, run_slide, SlideResult};
use std::env;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};

fn main_impl() -> Result<(), Box<dyn std::error::Error>> {
    let mut ch_stdout = BufferedStandardStream::stdout(ColorChoice::Auto);
    let mut ch_stderr = BufferedStandardStream::stderr(ColorChoice::Auto);
    let is_tty = atty::is(atty::Stream::Stderr);
    let use_color = is_tty && ch_stderr.supports_color();

    let opts = get_opts(|args| Ok(args.get_matches()), use_color).unwrap();
    let SlideResult {
        code,
        stdout,
        stderr,
        page,
    } = run_slide(opts);

    if !stderr.is_empty() {
        writeln!(&mut ch_stderr, "{}", stderr)?;
        ch_stderr.flush()?;
    }
    if !stdout.is_empty() {
        print_stdout(&stdout, &mut ch_stdout, page)?;
    }

    std::process::exit(code)
}

/// Basically just copied from rust/src/librustc_driver/lib.rs#show_content_with_pager
fn print_stdout(
    stdout: &str,
    mut ch_stdout: &mut BufferedStandardStream,
    page: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut fallback_to_println = false;

    if page {
        let pager_name = env::var_os("PAGER")
            .unwrap_or_else(|| OsString::from(if cfg!(windows) { "more.com" } else { "less" }));

        match Command::new(pager_name).stdin(Stdio::piped()).spawn() {
            Ok(mut pager) => {
                if let Some(pipe) = pager.stdin.as_mut() {
                    if pipe.write_all(stdout.as_bytes()).is_err() {
                        fallback_to_println = true;
                    }
                }

                if pager.wait().is_err() {
                    fallback_to_println = true;
                }
            }
            Err(_) => {
                fallback_to_println = true;
            }
        }
    }

    // If pager fails for whatever reason, we should still print the content to standard output.
    if fallback_to_println || !page {
        writeln!(&mut ch_stdout, "{}", stdout)?;
        ch_stdout.flush()?;
    }

    Ok(())
}

fn main() {
    let out = std::panic::catch_unwind(main_impl);

    if let Err(..) = out {
        eprint!(
            "\nnote: you found an internal slide error (ISE; it's like an ICE, but for slide)!\n"
        );
        eprint!("\nnote: we would appreciate a bug report: https://github.com/yslide/slide\n");
        std::process::exit(2);
    }
}

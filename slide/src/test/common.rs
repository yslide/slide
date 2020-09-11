use lazy_static::lazy_static;
use libtest_mimic::LinePrinter;
use std::collections::HashMap;
use std::sync::RwLock;

pub type ErrorMsgRef = usize;
pub type Printer = dyn Fn(&mut dyn LinePrinter) + Send + Sync;

lazy_static! {
    pub static ref TEST_CONSTRUCTION_FAIL: RwLock<HashMap<super::ErrorMsgRef, Box<super::Printer>>> =
        RwLock::new(HashMap::new());
    pub static ref BLESS: bool = std::env::var("BLESS") == Ok("1".into());
    pub static ref FAIL_TODO: bool = std::env::var("FAIL_TODO") == Ok("1".into());
}

macro_rules! prefix_severity {
    (Suggestion, $content:expr) => {
        format!("Hint: {}", $content)
    };

    ($other:ident, $content:expr) => {
        $content
    };
}

macro_rules! print_fail {
    ($($severity:ident: $($content:expr),*;)*) => {
        fail! { move |printer: &mut dyn libtest_mimic::LinePrinter| { printer! { printer
            $($severity: $($content),*;)*
        };}};
    };
}

macro_rules! printer {
    ($printer:ident $($severity:ident: $($content:expr),*;)*) => {$(
        $printer.print_line(
            &prefix_severity!($severity, &format!($($content),*)),
            &libtest_mimic::LineFormat::$severity
        );
    )*};
}

macro_rules! fail {
    ($report:expr) => {
        libtest_mimic::Outcome::Failed {
            msg: Some(atomic_lock($report)),
        }
    };
}

macro_rules! real_path {
    ($path:expr) => {
        format!("slide/{}", $path)
    };
}

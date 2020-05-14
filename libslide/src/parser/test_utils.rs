#![allow(unused_macros)]
macro_rules! __parse {
    ($parser:ident, $inout:expr) => {
        let inout: Vec<&str> = $inout.split(" => ").collect();
        let pin = inout[0];
        let pout = if inout.len() > 1 {
            inout[1].to_owned()
        } else {
            pin.to_owned()
        };
        let tokens = scan(pin);
        let (parsed, _) = $parser(tokens);
        assert_eq!(parsed.to_string(), pout);
    };
}

macro_rules! common_parser_tests {
    ($($name:ident: $inout:expr)*) => {
    $(
        #[test]
        fn $name() {
            use crate::scanner::{scan};
            use crate::parser::{parse_expression, parse_expression_pattern};

            __parse!(parse_expression, $inout);
            __parse!(parse_expression_pattern, $inout);
        }
    )*
    }
}

macro_rules! parser_tests {
    ($parser:ident $($name:ident: $program:expr)*) => {
    $(
        #[test]
        fn $name() {
            use crate::scanner::{scan};
            use crate::parser::{$parser};

            __parse!($parser, $program);
        }
    )*
    }
}

macro_rules! parser_error_tests {
    ($parser:ident $($name:ident: $program:expr => $error:expr)*) => {
    $(
        #[test]
        fn $name() {
            use crate::scanner::{scan};
            use crate::parser::{$parser};

            let tokens = scan($program);
            let (_, errors) = $parser(tokens);
            assert_eq!(errors.join("\n"), $error);
        }
    )*
    }
}

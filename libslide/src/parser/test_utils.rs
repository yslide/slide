#![allow(unused_macros)]
macro_rules! __parse {
    ($parser:ident, $program:expr) => {
        let tokens = scan($program);
        let (parsed, _) = $parser(tokens);
        assert_eq!(parsed.to_string(), $program);
    };
}

macro_rules! common_parser_tests {
    ($($name:ident: $program:expr)*) => {
    $(
        #[test]
        fn $name() {
            use crate::scanner::{scan};
            use crate::parser::{parse_expression, parse_expression_pattern};

            __parse!(parse_expression, $program);
            __parse!(parse_expression_pattern, $program);
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

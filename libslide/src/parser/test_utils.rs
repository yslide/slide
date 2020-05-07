#![allow(unused_macros)]
macro_rules! common_parser_tests {
    ($($name:ident: $program:expr)*) => {
    $(
        #[test]
        fn $name() {
            use crate::scanner::{scan};
            use crate::parser::{parse, ParsingStrategy};
            use ParsingStrategy::*;

            let stategies = vec![Expression, ExpressionPattern];

            for strategy in stategies.into_iter() {
                let tokens = scan($program);
                let (parsed, _) = parse(tokens, strategy);
                assert_eq!(parsed.to_string(), $program);
            }
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
            use crate::parser::{parse, ParsingStrategy};
            use ParsingStrategy::*;

            let tokens = scan($program);
            let (parsed, _) = parse(tokens, $parser);
            assert_eq!(parsed.to_string(), $program);
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
            use crate::parser::{parse, ParsingStrategy};
            use ParsingStrategy::*;

            let tokens = scan($program);
            let (_, errors) = parse(tokens, $parser);
            assert_eq!(errors.join("\n"), $error);
        }
    )*
    }
}

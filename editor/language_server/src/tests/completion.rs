use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! completions_tests {
     ($($name:ident: $text:expr)*) => {$(
         #[tokio::test]
         async fn $name() {
             let mut service = MockService::default().await;
             let file = default_file();

             let DecorationResult { mut decorations, cursor, text } = process_decorations($text);
             service.did_open(&file, &text).await;

             if decorations.len() > 1 {
                 panic!("Expected at most one completions decoration.");
             }
             let expected_completions: Option<Vec<String>> = decorations
                 .pop()
                 .and_then(|d| d.1)
                 .map(|c| c.split(",").map(ToString::to_string).collect());

             let (mut real, expected) = match (service.completion(&file, cursor.expect("No cursor found")).await, expected_completions) {
                 (Some(CompletionResponse::Array(real)), Some(expected)) => (real, expected),
                 (None, None) => {
                     return;
                 }
                 _ => unreachable!("Actual and expected completions don't match"),
             };
             real.sort_by(|a, b| a.label.cmp(&b.label));

             assert_eq!(real.len(), expected.len());
             for (real, expected) in real.into_iter().zip(expected) {
                 assert_eq!(real.label, expected);
             }

             service.shutdown().await;
         }
     )*}
 }

completions_tests! {
    var_completions_in_var: r"
        a := a + a¦b
                   ~@[a,c]
        c := 1 + 2
    "
    var_completions_start_of_var: r"
        a := a + ¦ab
                  ~@[a,c]
        c := 1 + 2
    "
    var_completions_end_of_var: r"
        a := a + ab¦
                    ~@[a,c]
        c := 1 + 2
    "
    // TODO: currently only "c" and "a" are returned because "d" is parsed as part of the addition
    // var_completions_in_incomplete: r"
    //      c := 1 + 2
    //      a := a + ¦
    //                ~@[a,c,d]
    //      d := 1 + 2
    //  "
}

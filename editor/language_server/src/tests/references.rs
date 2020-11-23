use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! references_tests {
    ($($name:ident: $include_declaration:expr, $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let expected_ranges = decorations
                .into_iter()
                .map(|(r, _)| Location::new(file.clone(), r))
                .collect::<Vec<_>>();
            let expected_ranges = if expected_ranges.is_empty() { None } else { Some(expected_ranges) };

            let references = service.references(&file, cursor.expect("cursor not found"), $include_declaration).await;

            assert_eq!(references, expected_ranges);

            service.shutdown().await;
        }
    )*}
}

references_tests! {
    var_no_declaration: false, r"
        a := ¦a + b
              ~
        a := a + c + a
             ~       ~
    "
    var_all_references: true, r"
        a := ¦a + b
        ~     ~
        a := a + c + a
        ~    ~       ~
    "
    no_references: true, r"
        a := ¦1
        a := a + c + a
    "
}

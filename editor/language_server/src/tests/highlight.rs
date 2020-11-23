use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! references_tests {
    ($($name:ident: $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let expected_ranges = decorations
                .into_iter()
                .map(|(range, cmt)| {
                    let kind = Some(match cmt.expect("Expected highlight kind").as_ref() {
                        "R" => DocumentHighlightKind::Read,
                        "W" => DocumentHighlightKind::Write,
                        els => panic!("Invalid highlight kind {}", els),
                    });
                    DocumentHighlight { range, kind }
                })
                .collect::<Vec<_>>();
            let expected_ranges = if expected_ranges.is_empty() { None } else { Some(expected_ranges) };

            let references = service.highlight(&file, cursor.expect("cursor not found")).await;

            assert_eq!(references, expected_ranges);

            service.shutdown().await;
        }
    )*}
}

references_tests! {
    var_highlights: r"
        a := ¦a + b
        ~@[W] ~@[R]
        a := 1 + a + c + a
        ~@[W]    ~@[R]   ~@[R]
    "
    no_highlights: r"
        a := ¦1
        a := a + c + a
    "
}

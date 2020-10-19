use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! completions_tests {
    ($($name:ident: $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = Url::parse("file:///test").unwrap();

            let DecorationResult { mut decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            if decorations.len() > 1 {
                panic!("Expected at most one completions decoration.");
            }
            let expected_completions = decorations
                .pop()
                .and_then(|d| d.1)
                .map(|c| CompletionResponse::Array(
                        c
                        .split(",")
                        .map(|c| CompletionItem {
                            label: c.to_string(),
                            kind: Some(CompletionItemKind::Variable),
                            insert_text_format: Some(InsertTextFormat::PlainText),
                            ..CompletionItem::default()
                        })
                        .collect()
                    )
                );

            let completions = service.completion(&file, cursor).await;

            assert_eq!(completions, expected_completions);

            service.shutdown().await;
        }
    )*}
}

completions_tests! {
    var_completions: r"
        a := a + a¦b
                  ~@[a,c]
        c := 1 + 2
    "
    var_completions_in_incomplete: r"
        a := a + ¦
                 ~@[a,c,d]
        c := 1 + 2
    "
    no_completions: r"
        a := ¦1
        a := a + c + a
    "
}

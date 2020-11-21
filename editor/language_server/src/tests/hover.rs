use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! hover_tests {
    ($($name:ident: $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let hover_info = match service.hover(&file, cursor).await {
                Some(hover) => hover,
                None => {
                    assert!(decorations.is_empty(), "Expected no hover contents!");
                    return;
                }
            };

            let (expected_range, expected_content) = decorations.into_iter().next().unwrap();
            let expected_content = HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "slide".to_string(),
                value: expected_content.expect("Expected hover contents!"),
            }));

            assert_eq!(hover_info.range, Some(expected_range));
            assert_eq!(hover_info.contents, expected_content);

            service.shutdown().await;
        }
    )*}
}

hover_tests! {
    simple_expr: r"
        a := ¦1 + 2
              ~@[= 1]"
    binary_operator: r"
        a := 1 ¦+ 2
             ~~~~~~@[= 3]"
    inside_binary_expression: r"
        a := 1 ¦ + 2
             ~~~~~~~@[= 3]"
    unary_expression: r"
        a := ¦++2
              ~~~@[= 2]"
    paren: r"
        ¦(1 + 5)
         ~~~~~~~@[= 6]"
    var: r"
        ¦a := 5 + 6
         ~@[= 11]"
    unknown_var: r"
        a := ¦c
              ~@[= ???]"
    multidefined_var: r"
        ¦a := c
         ~@[= c<|>= c * 2]
         a := 2c
    "
    no_hover: r"
        a :¦= b
    "
}

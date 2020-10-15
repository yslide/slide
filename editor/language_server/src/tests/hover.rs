use super::mocks::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! hover_tests {
    ($($name:ident: $text:expr => $range:expr, $content:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::new().await;
            let file = Url::parse("file:///test").unwrap();

            let (text, pos) = get_hover($text);
            service.did_open(&file, &text).await;

            let hover_info = service.hover(&file, pos).await;
            let range = range_of($range, &text);
            let contents = HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "slide".to_string(),
                value: $content.to_string(),
            }));

            assert_eq!(hover_info.range, Some(range));
            assert_eq!(hover_info.contents, contents);

            service.shutdown().await;
        }
    )*}
}

hover_tests! {
    simple_expr: "a := ¦1 + 2" => "1", "= 1"
    binary_operator: "a := 1 ¦+ 2" => "1 + 2", "= 3"
    inside_binary_expression: "a := 1 ¦ + 2" => "1  + 2", "= 3"
    unary_expression: "a := ¦++2" => "++2", "= 2"
    paren: "¦(1 + 5)" => "(1 + 5)", "= 6"
    var: "¦a := 5 + 6" => "a", "= 11"
    unknown_var: "a := ¦c" => "c", "= ???"
    multidefined_var: r#"
        ¦a := c
         a := 2c
        "# => "a", "= c\n= c * 2"
}

/// Pre-processes text with the hover marker ¦,
/// returning the text with the hover marker removed
/// and the [`Position`](Position) of the marker.
///
/// The returned `Position` always marks the character to right of its original location,
/// for example in "ab¦cde",
/// the hover position would be over "c".
fn get_hover(text: &str) -> (String, Position) {
    let mut cursors = text.match_indices('¦');
    let cursor = cursors.next().expect("Hover cursor must be present.").0;
    if cursors.next().is_some() {
        panic!("Cannot have more than two hover cursors.");
    }

    let position = crate::shims::to_position(cursor, text);
    let text = text.split('¦').collect::<Vec<_>>().join("");
    (text, position)
}

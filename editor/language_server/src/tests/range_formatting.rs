use super::mocks::*;
use crate::document_registry::SourceMap;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_range_formatting_test(
    orig: &str,
    expected_edit: Option<&str>,
    service: &mut MockService,
    file: &Url,
    content: &str,
    sm: &SourceMap,
) {
    let start = content
        .find(orig)
        .unwrap_or_else(|| panic!("{} not found", orig));
    let end = start + orig.len();
    let range = Range::new(sm.to_position(start), sm.to_position(end));
    match (service.range_formatting(&file, &range).await, expected_edit) {
        (None, None) => {}
        (Some(mut edits), Some(expected_edit)) => {
            if edits.len() != 1 {
                panic!("Expected only one edit!");
            }
            let edit = edits.pop().unwrap();
            assert_eq!(edit.range, range);
            assert_eq!(edit.new_text, expected_edit);
        }
        (actual, expected) => panic!(
            "Actual ({:?}) and expected ({:?}) edits don't match.",
            actual, expected
        ),
    };
}

#[tokio::test]
async fn range_formatting() {
    let parsers = markdown_math_document_parsers();
    let mut service = MockService::new(/* link support */ false, parsers).await;
    let file = markdown_file();
    let content = r"
# Hello

```math
1    + 2
 / 3
  ^ 4
```

```math
a     = b +   c
def  :=   234
  * 78
```
";
    service.did_open(&file, content).await;
    let sm = SourceMap::new(content);

    macro_rules! wrap_test {
        ($service:ident, $file:ident, $content:ident, $sm:ident, $($orig:expr => $expected:expr)*) => {$(
            drive_range_formatting_test($orig, $expected, &mut $service, &$file, $content, &$sm).await;
        )*}
    }

    wrap_test! {
        service, file, content, sm,
        "1    + 2\n / 3\n  ^ 4" => Some("1 + 2 / 3 ^ 4")
        "1    + 2"              => None
        "3"                     => Some("3")
        " 3"                    => None
        "a     = b +   c"       => Some("a = b + c")
        "b +   c"               => Some("b + c")
        " b +   c"              => None
        "def  :=   234\n  * 78" => Some("def := 234 * 78")
        "234\n  * 78"           => Some("234 * 78")
        "def  :=   234"         => None
    }

    service.shutdown().await;
}

use super::mocks::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_formatting_test(content: &str, expected_edits: &[(Position, Position, &str)]) {
    let parsers = markdown_math_document_parsers();
    let mut service = MockService::new(/* link support */ false, parsers).await;
    let file = markdown_file();

    service.did_open(&file, content).await;

    let edits = service.formatting(&file).await.unwrap();
    for (expected_edit, edit) in expected_edits.iter().zip(edits) {
        let &(start, end, expected_edit) = expected_edit;
        assert_eq!(edit.range.start, start);
        assert_eq!(edit.range.end, end);
        assert_eq!(edit.new_text, expected_edit);
    }

    service.shutdown().await;
}

#[tokio::test]
async fn formatting() {
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
    let expected_edits = &[
        (Position::new(4, 0), Position::new(6, 5), "1 + 2 / 3 ^ 4"),
        (
            Position::new(10, 0),
            Position::new(12, 6),
            "a = b + c\ndef := 234 * 78",
        ),
    ];
    drive_formatting_test(content, expected_edits).await;
}

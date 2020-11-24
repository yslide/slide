use super::mocks::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_folding_ranges_tests(content: &str, expected_ranges: &[Range]) {
    let parsers = markdown_math_document_parsers();
    let mut service = MockService::new(/* link support */ false, parsers).await;
    let file = markdown_file();

    service.did_open(&file, content).await;

    let folding_ranges = service.folding_range(&file).await.unwrap();
    for (range, expected_range) in folding_ranges.iter().zip(expected_ranges) {
        assert_eq!(range.start_line, expected_range.start.line);
        assert_eq!(range.start_character, Some(expected_range.start.character));
        assert_eq!(range.end_line, expected_range.end.line);
        assert_eq!(range.end_character, Some(expected_range.end.character));
    }

    service.shutdown().await;
}

#[tokio::test]
async fn folding_range() {
    let content = r"
# Hello

```math
1    + 2
 / 3
  ^ 4
d := g + h
 /
   10
```

```math
a     = b +   c

def  :=   234

  * 78
```
";
    let expected_ranges = &[
        Range::new(Position::new(4, 0), Position::new(6, 5)),
        Range::new(Position::new(7, 0), Position::new(9, 5)),
        Range::new(Position::new(13, 0), Position::new(13, 15)),
        Range::new(Position::new(15, 0), Position::new(17, 6)),
    ];
    drive_folding_ranges_tests(content, expected_ranges).await;
}

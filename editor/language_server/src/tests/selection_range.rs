use super::mocks::*;
use crate::document_registry::SourceMap;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_selection_ranges_test(
    content: &str,
    positions: &[Position],
    expected_ranges: &Option<Vec<SelectionRange>>,
) {
    let mut service = MockService::default().await;
    let file = default_file();

    service.did_open(&file, content).await;

    let selection_ranges = service.selection_range(&file, positions).await;
    assert_eq!(&selection_ranges, expected_ranges);

    service.shutdown().await;
}

#[tokio::test]
async fn selection_range() {
    let content = r"
a := 1 + 2 / 5  
";
    let sm = SourceMap::new(content);
    let range = |over: &str| {
        let start = content.find(over).unwrap();
        Range::new(sm.to_position(start), sm.to_position(start + over.len()))
    };

    let pos_2 = sm.to_position(content.find('2').unwrap());
    let selection_range_2 = SelectionRange {
        range: range("2"),
        parent: Some(Box::new(SelectionRange {
            range: range("2 / 5"),
            parent: Some(Box::new(SelectionRange {
                range: range("1 + 2 / 5"),
                parent: Some(Box::new(SelectionRange {
                    range: range("a := 1 + 2 / 5"),
                    parent: None,
                })),
            })),
        })),
    };
    let pos_end = Position::new(1, 16);

    let tests = &[
        (vec![pos_2], Some(vec![selection_range_2])),
        // pos_end has no selection range, so nothing should be returned because otherwise the
        // condition that selection_range(request[i]) = response[i] would be broken.
        (vec![pos_end], None),
        (vec![pos_2, pos_end], None),
    ];

    for (positions, expected_ranges) in tests {
        drive_selection_ranges_test(content, positions, expected_ranges).await;
    }
}

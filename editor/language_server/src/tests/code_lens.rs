use super::mocks::*;
use crate::document_registry::SourceMap;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_code_lens_test(content: &str, ranges: &[(Range, &str)]) {
    let mut service = MockService::default().await;
    let file = default_file();
    service.did_open(&file, content).await;

    let code_lenses = service.code_lens(&file).await.unwrap();
    for (cl, (range, simpl)) in code_lenses.into_iter().zip(ranges) {
        assert_eq!(cl.range, *range);
        assert_eq!(
            cl.data
                .map(|v| match v {
                    serde_json::Value::String(s) => s,
                    _ => Default::default(),
                })
                .unwrap_or_default(),
            *simpl
        );
    }

    service.shutdown().await;
}

#[tokio::test]
async fn code_lenses() {
    let content = r"
1 + 2 / 4
2 + 3
";
    let sm = SourceMap::new(content);
    let range = |over: &str| {
        let start = content.find(over).unwrap();
        Range::new(sm.to_position(start), sm.to_position(start + over.len()))
    };

    let lenses_ranges = [
        (range("2 / 4"), "0.5"),
        (range("1 + 2 / 4"), "1.5"),
        (range("2 + 3"), "5"),
    ];

    drive_code_lens_test(content, &lenses_ranges).await;
}

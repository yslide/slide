use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! prepare_rename_tests {
    ($($name:ident: $text:expr, $status:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let status: Result<(), &'static str> = $status;
            let mut service = MockService::default().await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let prepare_rename_response = match (service.prepare_rename(&file, &cursor.expect("cursor not found")).await, status) {
                (Ok(resp), Ok(_)) => resp.unwrap(),
                (Err(err), Err(msg)) => {
                    assert_eq!(err.message, msg);
                    return;
                }
                _ => panic!("Expected and actual responses differ!")
            };

            let (expected_range, expected_placeholder) = decorations.into_iter().next().unwrap();
            let expected_prepare_rename_response = PrepareRenameResponse::RangeWithPlaceholder {
                range: expected_range,
                placeholder: expected_placeholder.expect("no placeholder provided"),
            };

            assert_eq!(prepare_rename_response, expected_prepare_rename_response);

            service.shutdown().await;
        }
    )*}
}

prepare_rename_tests! {
    no_item_1: r"a := ¦1 + 2", Err("cursor is not over a variable")
    no_item_2: r"a := 1 ¦+ 2", Err("cursor is not over a variable")
    no_item_3: r"a :¦= 1 + 2", Err("cursor is not over a variable")
    variable_at_definition: r"
        a¦bc := 1 + 2
        ~~~~@[new variable]
        ", Ok(())
    variable_in_expression: r"
        1 + a¦bc + 2
            ~~~~@[new variable]
        ", Ok(())
}

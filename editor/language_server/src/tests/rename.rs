use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;

macro_rules! rename_tests {
    ($($name:ident => $new_name:expr, $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let expected_edits = decorations
                .into_iter()
                .map(|(range, rename)| TextEdit {range, new_text: rename.expect("no rename given")})
                .collect::<Vec<_>>();
            let expected_edits = if expected_edits.is_empty() { None } else {
                let mut changes = HashMap::new();
                changes.insert(file.clone(), expected_edits);
                Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                })
            };

            let edits = service.rename(&file, &cursor.expect("cursor not found"), $new_name).await;

            assert_eq!(edits, expected_edits);

            service.shutdown().await;
        }
    )*}
}

rename_tests! {
    no_rename_1 => "abc", r"
        a := a ¦+ b
        a := a + c + a
    "
    no_rename_2 => "abc", r"
        a := ¦1
        1 + 2
    "
    rename_var_in_assignment => "abc", r"
        ¦ad := 2 + ad
        ~~~@[abc]  ~~@[abc]
        def := ad ^ 3 + ad
               ~~@[abc] ~~@[abc]
        ab + ad
             ~~@[abc]
    "
    rename_var_in_expression => "f", r"
        ¦a + b + c
        ~~@[f]
        d * e ^ a
                ~@[f]
    "
}

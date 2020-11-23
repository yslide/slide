use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! definitions_tests {
    ($($name:ident, $link_support:expr, $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::new($link_support, default_initialization_options()).await;
            let file = default_file();

            let DecorationResult { decorations, cursor, text } = process_decorations($text);
            service.did_open(&file, &text).await;

            let origin_selection_range = decorations.iter()
                .find(|(_, c)| c.as_deref() == Some("origin"))
                .map(|(r, _)| r.clone());
            let expected_defs = if $link_support {
                GotoDefinitionResponse::Link(decorations
                    .into_iter()
                    .filter_map(|(r, c)| {
                        if c.is_some() { return None; }
                        Some(LocationLink {
                            origin_selection_range,
                            target_uri: file.clone(),
                            target_range: r,
                            target_selection_range: r,
                        })
                    })
                    .collect::<Vec<_>>()
                )
            } else {
                GotoDefinitionResponse::Array(decorations
                    .into_iter()
                    .map(|(r, _)| {
                        Location {
                            uri: file.clone(),
                            range: r,
                        }
                    })
                    .collect::<Vec<_>>()
                )
            };
            let expected_defs = match expected_defs {
                GotoDefinitionResponse::Link(defs) if defs.is_empty() => None,
                GotoDefinitionResponse::Array(defs) if defs.is_empty() => None,
                _ => Some(expected_defs),
            };

            let defs = service.definition(&file, cursor.expect("cursor not found")).await;

            assert_eq!(defs, expected_defs);

            service.shutdown().await;
        }
    )*}
}

definitions_tests! {
    var_definitions_links, true, r"
        abc := b + c
        ~~~
        abc := a + c + a
        ~~~
        def := a¦bc + abc
               ~~~~@[origin]
    "
    var_definitions_locations, false, r"
        abc := b + c
        ~~~
        abc := a + c + a
        ~~~
        def := a¦bc + abc
    "
    no_var_definitions, true, r"
        abc := b + c
        abc := ¦a + c + a
    "
    no_definitions, true, r"
        a := ¦1 + 2 + 3
    "
}

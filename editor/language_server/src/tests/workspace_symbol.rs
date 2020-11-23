use super::mocks::*;
use super::utils::*;

use tower_lsp::lsp_types::*;

macro_rules! workspace_symbol_tests {
    ($($name:ident: query $query:expr => $($file:expr => $text:expr),*)*) => {$(
        #[tokio::test]
        async fn $name() {
            let document_parsers = markdown_math_document_parsers();
            let mut service = MockService::new(/* link_support */ false, document_parsers).await;

            let mut expected_symbols = vec![];
            $({
                let file = Url::parse($file).unwrap();
                let DecorationResult { decorations, text, .. } = process_decorations($text);
                service.did_open(&file, &text).await;

                let parsed_symbols = decorations.into_iter().map(|(range, content)| {
                    let content = content.unwrap();
                    let mut content = content.split(',');
                    #[allow(deprecated)]
                    SymbolInformation {
                        name: content.next().unwrap().to_owned(),
                        kind: match content.next().unwrap().as_ref() {
                            "variable" => SymbolKind::Variable,
                            _ => unreachable!(),
                        },
                        location: Location {
                            uri: file.clone(),
                            range,
                        },
                        deprecated: None,
                        container_name: None,
                    }
                });
                expected_symbols.extend(parsed_symbols);
            })*

            let workspace_symbol_info = match service.workspace_symbol($query).await {
                Some(ws_symbols) => ws_symbols,
                None => {
                    assert!(expected_symbols.is_empty(), "Expected no symbols!");
                    return;
                }
            };

            cmp_symbols(workspace_symbol_info, expected_symbols);

            service.shutdown().await;
        }
    )*}
}

workspace_symbol_tests! {
    symbols_in_workspace_empty_query:
    query "" =>
    "file:///fi1.md" => r"
# Hello world

```math
a = 1 + 2
~@[a,variable]
b = 2 + 3
~@[b,variable]
```
",
    "file:///fi2.md" => r"
# Othello

```math
dd = a + b
~~@[dd,variable]
1 + 3
ee = c + d
~~@[ee,variable]
```

`a + d` or

```math
a + d
```
"

    symbols_in_workspace_non_empty_query:
    query "abc" =>
    "file:///fi1.md" => r"
# Hello world

```math
abcde = 1 + 2
~~~~~@[abcde,variable]
bcda = 2 + 3
```
",
    "file:///fi2.md" => r"
# Othello

```math
fabcd = a + b
~~~~~@[fabcd,variable]
1 + 3
ee = c + d
abc = 1
~~~@[abc,variable]
ab = 3
```
"
}

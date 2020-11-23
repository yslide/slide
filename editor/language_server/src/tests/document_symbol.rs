use super::mocks::*;
use super::utils::*;

use tower_lsp::lsp_types::*;

macro_rules! document_symbol_tests {
    ($($name:ident: $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let document_parsers = markdown_math_document_parsers();
            let mut service = MockService::new(/* link_support */ false, document_parsers).await;
            let file = markdown_file();

            let DecorationResult { decorations, text, .. } = process_decorations($text);
            service.did_open(&file, &text).await;

            let document_symbol_info = match service.document_symbol(&file).await {
                Some(doc_symbols) => doc_symbols,
                None => {
                    assert!(decorations.is_empty(), "Expected no symbols!");
                    return;
                }
            };

            let expected_symbols = DocumentSymbolResponse::Flat(decorations.into_iter().map(|(range, content)| {
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
            }).collect());

            cmp_document_symbols(document_symbol_info, expected_symbols);

            service.shutdown().await;
        }
    )*}
}

document_symbol_tests! {
    symbols_in_document: r"
# Hello world

```math
a = 1 + 2
~@[a,variable]
b = 2 + 3
~@[b,variable]
```

## Othello

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
}

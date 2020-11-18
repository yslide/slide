use super::mocks::*;
use super::utils::range_of;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

macro_rules! related_info {
    ($text:ident, $($file:ident@$range:expr, $msg:expr),*) => {
        vec![$(
            DiagnosticRelatedInformation {
                location: Location {
                    uri: $file.clone(),
                    range: range_of($range, &$text),
                },
                message: $msg.to_string(),
            }
        ),*]
    }
}

macro_rules! diagnostics {
    ($text:ident, $($range:expr, [$severity:ident $code:ident] $msg:expr ;; $related_info:expr),*) => {
        vec![$(
            Diagnostic {
                range: range_of($range, &$text),
                severity: Some(DiagnosticSeverity::$severity),
                code: Some(NumberOrString::String(stringify!($code).to_string())),
                source: Some("slide".to_string()),
                message: $msg.to_string(),
                related_information: Some($related_info),
                tags: None,
            }
        ),*]
    }
}

#[tokio::test]
async fn empty_diagnostics() {
    let mut service = MockService::default().await;

    let file = Url::parse("file:///test").unwrap();
    let text = r#"
    a := 1 + 2
    b := a + 5
    "#;

    let diagnostics = service.did_open(&file, text).await;

    assert_eq!(diagnostics.uri, file);
    assert!(diagnostics.diagnostics.is_empty());

    service.shutdown().await;
}

#[tokio::test]
async fn open_and_change_with_diagnostics() {
    let mut service = MockService::default().await;

    let file = Url::parse("file:///test").unwrap();
    let text = r#"
    a := 1 + 2
    c := 5 + ++5 + /
    "#;

    let diagnostics = service.did_open(&file, text).await;

    assert_eq!(diagnostics.uri, file);
    assert_eq!(
        diagnostics.diagnostics,
        diagnostics! {
            text,
            "/", [Error P0002] "Expected an expression, found / \\ expected an expression";; vec![],
            "++5", [Warning L0002] "Trivially reducible unary operator chain";; related_info! { text,
            }
        }
    );

    // Now change the file to fix one diagnostic.
    let text = r#"
    a := 1 + 2
    c := 5 + ++5
    "#;

    let diagnostics = service.did_change(&file, text).await;

    assert_eq!(diagnostics.uri, file);
    assert_eq!(
        diagnostics.diagnostics,
        diagnostics! {
            text,
            "++5", [Warning L0002] "Trivially reducible unary operator chain";; related_info! { text,
            }
        }
    );

    // Now fix the only other diagnostic.
    let text = r#"
    a := 1 + 2
    "#;

    let diagnostics = service.did_change(&file, text).await;

    assert_eq!(diagnostics.uri, file);
    assert!(diagnostics.diagnostics.is_empty());

    service.shutdown().await;
}

use super::mocks::*;
use super::utils::*;

use pretty_assertions::assert_eq;
use std::collections::HashSet;
use tower_lsp::lsp_types::*;

#[derive(Debug, PartialEq)]
struct PartialAction {
    title: String,
    diagnostic_codes: HashSet<String>,
}
impl PartialAction {
    fn from_action(action: CodeActionOrCommand) -> PartialAction {
        match action {
            CodeActionOrCommand::Command(_) => {
                unreachable!("All actions should be code actions!");
            }
            CodeActionOrCommand::CodeAction(ca) => PartialAction {
                title: ca.title,
                diagnostic_codes: ca
                    .diagnostics
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| {
                        d.code
                            .map(|c| match c {
                                NumberOrString::Number(n) => n.to_string(),
                                NumberOrString::String(s) => s,
                            })
                            .unwrap_or_default()
                    })
                    .collect(),
            },
        }
    }
}

fn parse_actions(actions: String) -> Vec<PartialAction> {
    let re_action = regex::Regex::new(r"(?P<diags>\(.*\))?(?P<title>.+)").unwrap();
    re_action
        .captures_iter(&actions)
        .map(|cap| {
            let title = cap.name("title").unwrap().as_str().to_owned();
            let diagnostic_codes = cap
                .name("diags")
                .map(|s| {
                    s.as_str()
                        .split(",")
                        .map(|code| code.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default();
            PartialAction {
                title,
                diagnostic_codes,
            }
        })
        .collect()
}

macro_rules! actions_tests {
    ($($name:ident: $text:expr)*) => {$(
        #[tokio::test]
        async fn $name() {
            let mut service = MockService::default().await;
            let file = Url::parse("file:///test").unwrap();

            let DecorationResult { mut decorations, text, .. } = process_decorations($text);
            let PublishDiagnosticsParams { diagnostics, .. } = service.did_open(&file, &text).await;

            if decorations.len() != 1 {
                panic!("Expected exactly one location for actions!");
            }
            let (range, unparsed_actions) = decorations.pop().unwrap();
            let expected_actions = parse_actions(unparsed_actions.unwrap_or_default());

            let real_actions = match service.code_action(&file, range, diagnostics).await {
                Some(actions) => actions,
                None => {
                    assert!(expected_actions.is_empty(), "Expected no actions!");
                    return;
                }
            };

            let real_actions = real_actions.into_iter().map(PartialAction::from_action).collect::<Vec<_>>();

            assert_eq!(expected_actions, real_actions);

            service.shutdown().await;
        }
    )*}
}

actions_tests! {
    const_expr: r"
        a := 1 + 2
             ~@[]"
    var_expr: r"
        a := 1 + 2
        ~@[]"
    binary_expr: r"
        a := 1 + 2
             ~~~~~@[Simplify expression]"
    unary_expr: r"
        a := ++2
             ~~~@[Simplify expression]"
    paren_expr: r"
        (1 + 5)
        ~~~~~~~@[Simplify expression]"
    bracketed_expr: r"
        [1 + 5]
        ~~~~~~~@[Simplify expression]"
}

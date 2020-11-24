use super::mocks::*;
use crate::document_registry::SourceMap;

use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::*;

async fn drive_code_action_test(content: &str, range: &Range, check: impl FnOnce(Vec<CodeAction>)) {
    let mut service = MockService::default().await;
    let file = default_file();

    service.did_open(&file, content).await;

    let actions = service.code_action(&file, range).await;
    let actions: Vec<_> = actions
        .map(|a| {
            a.into_iter()
                .map(|a| match a {
                    CodeActionOrCommand::CodeAction(a) => a,
                    _ => unreachable!(),
                })
                .collect()
        })
        .unwrap();
    check(actions);

    service.shutdown().await;
}

#[tokio::test]
async fn code_actions() {
    let content = r"
a := 1 + +++2 / 5  
";
    let sm = SourceMap::new(content);
    let range = |over: &str| {
        let start = content.find(over).unwrap();
        Range::new(sm.to_position(start), sm.to_position(start + over.len()))
    };
    let get_edit = |edit: Option<&WorkspaceEdit>| {
        edit.and_then(|e| e.changes.as_ref())
            .unwrap()
            .values()
            .next()
            .and_then(|e| e.last())
            .unwrap()
            .new_text
            .clone()
    };

    drive_code_action_test(content, &range("+++2"), |actions| {
        assert_eq!(actions.len(), 2);

        // Action resolving unary series diagnostic
        assert_eq!(actions[0].kind, Some(CodeActionKind::QUICKFIX));
        assert!(actions[0].diagnostics.is_some());
        assert_eq!(get_edit(actions[0].edit.as_ref()), "2");
        assert_eq!(actions[0].command, None);
        assert_eq!(actions[0].is_preferred, Some(true));

        // Generic rewrite action
        assert_eq!(actions[1].kind, Some(CodeActionKind::REFACTOR_REWRITE));
        assert!(actions[1].diagnostics.is_none());
        assert_eq!(get_edit(actions[1].edit.as_ref()), "2");
        assert_eq!(actions[1].command, None);
        assert_eq!(actions[1].is_preferred, Some(true));
    })
    .await;
}

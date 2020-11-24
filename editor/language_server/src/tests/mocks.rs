//! Mock service for language server testing.
//! Extended from https://github.com/wasm-lsp/wasm-language-server/blob/main/crates/testing/src/lsp.rs

use serde_json::Value;
use tokio::stream::StreamExt;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, MessageStream};
use tower_test::mock::Spawn;

pub fn default_file() -> Url {
    Url::parse("file:///fi.slide").unwrap()
}

pub fn default_initialization_options() -> Value {
    serde_json::json!({
        "document_parsers": {
            "slide": r"((?:.|\n)*)",
        },
    })
}

pub fn markdown_file() -> Url {
    Url::parse("file:///fi.md").unwrap()
}

pub fn markdown_math_document_parsers() -> Value {
    serde_json::json!({
        "document_parsers": {
            "md": r"```math\n((?:.|\n)*?)\n```",
        },
    })
}

pub struct MockService {
    service: Spawn<LspService>,
    msg_stream: MessageStream,
}

impl MockService {
    pub async fn default() -> Self {
        Self::new(false, default_initialization_options()).await
    }

    /// Creates a new slide language service and initializes it.
    pub async fn new(link_support: bool, initialization_options: Value) -> Self {
        let (service, msg_stream) = LspService::new(crate::SlideLS::new);
        let service = Spawn::new(service);
        let mut service = Self {
            service,
            msg_stream,
        };

        service.assert_ready();

        // Initialize
        service
            .send_recv(
                initialize::request(link_support, initialization_options),
                Some(initialize::response()),
            )
            .await;
        // Mark initialized
        service.send_recv(initialized::notification(), None).await;
        // Skip "server initialized" message
        service.msg_stream.next().await;

        service
    }

    pub async fn shutdown(&mut self) {
        self.assert_ready();
        self.send_recv(shutdown::request(), Some(shutdown::response()))
            .await;

        self.assert_ready();
        self.send_recv(exit::notification(), None).await;
    }

    fn assert_ready(&mut self) {
        assert_eq!(self.service.poll_ready(), std::task::Poll::Ready(Ok(())))
    }

    async fn send(&mut self, send: Value) -> Option<Value> {
        let request = serde_json::from_value(send).unwrap();
        let response = self.service.call(request).await.unwrap();
        response.and_then(|x| serde_json::to_value(x).ok())
    }

    async fn send_recv(&mut self, send: Value, recv: Option<Value>) {
        assert_eq!(self.send(send).await, recv);
    }

    async fn get_diagnostics(&mut self) -> PublishDiagnosticsParams {
        let diagnostics = self.msg_stream.next().await.unwrap();
        serde_json::from_value(
            serde_json::to_value(&diagnostics)
                .unwrap()
                .get("params")
                .unwrap()
                .clone(),
        )
        .unwrap()
    }

    pub async fn did_open(&mut self, uri: &Url, text: &str) -> PublishDiagnosticsParams {
        self.assert_ready();
        self.send_recv(
            text_document::did_open::notification(uri, "slide", 0, text),
            None,
        )
        .await;

        self.get_diagnostics().await
    }

    pub async fn did_change(&mut self, uri: &Url, text: &str) -> PublishDiagnosticsParams {
        self.assert_ready();
        self.send_recv(
            text_document::did_change::notification::entire(uri, text),
            None,
        )
        .await;

        self.get_diagnostics().await
    }

    pub async fn definition(
        &mut self,
        uri: &Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::definition::request(uri, position))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn hover(&mut self, uri: &Url, position: Position) -> Option<Hover> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::hover::request(uri, position))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn references(
        &mut self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
    ) -> Option<Vec<Location>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::references::request(
                uri,
                position,
                include_declaration,
            ))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn highlight(
        &mut self,
        uri: &Url,
        position: Position,
    ) -> Option<Vec<DocumentHighlight>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::highlight::request(uri, position))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn document_symbol(&mut self, uri: &Url) -> Option<DocumentSymbolResponse> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::document_symbol::request(uri))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn formatting(&mut self, uri: &Url) -> Option<Vec<TextEdit>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::formatting::request(uri))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn range_formatting(&mut self, uri: &Url, range: &Range) -> Option<Vec<TextEdit>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::range_formatting::request(uri, range))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn prepare_rename(
        &mut self,
        uri: &Url,
        position: &Position,
    ) -> tower_lsp::jsonrpc::Result<Option<PrepareRenameResponse>> {
        self.assert_ready();
        let resp = self
            .send(text_document::prepare_rename::request(uri, position))
            .await
            .unwrap();
        if let Some(result) = resp.get("result") {
            Ok(serde_json::from_value(result.clone()).ok())
        } else {
            Err(serde_json::from_value(resp.get("error").unwrap().clone()).unwrap())
        }
    }

    pub async fn rename(
        &mut self,
        uri: &Url,
        position: &Position,
        new_name: &str,
    ) -> Option<WorkspaceEdit> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::rename::request(uri, position, new_name))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn folding_range(&mut self, uri: &Url) -> Option<Vec<FoldingRange>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::folding_range::request(uri))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn selection_range(
        &mut self,
        uri: &Url,
        positions: &[Position],
    ) -> Option<Vec<SelectionRange>> {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::selection_range::request(uri, positions))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn code_action(&mut self, uri: &Url, range: &Range) -> Option<CodeActionResponse> {
        self.assert_ready();
        let action_resp = self
            .send(text_document::code_action::request(uri, range))
            .await
            .unwrap();
        serde_json::from_value(action_resp.get("result").unwrap().clone()).ok()
    }

    pub async fn workspace_symbol(&mut self, query: &str) -> Option<Vec<SymbolInformation>> {
        self.assert_ready();
        let hover_resp = self.send(workspace::symbol::request(query)).await.unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).ok()
    }
}

pub mod exit {
    use serde_json::{json, Value};

    pub fn notification() -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })
    }
}

pub mod initialize {
    use serde_json::{json, Value};

    pub fn request(link_support: bool, initialization_options: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "capabilities": {
                    "textDocument": {
                        "definition": {
                            "linkSupport": link_support,
                        },
                    },
                },
                "initializationOptions": initialization_options,
            },
            "id": 1,
        })
    }

    pub fn response() -> Value {
        json!({
            "jsonrpc": "2.0",
            "result": {
                "capabilities": crate::SlideLS::capabilities(),
            },
            "id": 1,
        })
    }
}

pub mod initialized {
    use serde_json::{json, Value};

    pub fn notification() -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        })
    }
}

pub mod shutdown {
    use serde_json::{json, Value};

    pub fn request() -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "shutdown",
            "id": 1,
        })
    }

    pub fn response() -> Value {
        json!({
            "jsonrpc": "2.0",
            "result": null,
            "id": 1,
        })
    }
}

pub mod text_document {
    pub mod did_change {

        pub mod notification {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn entire<S: AsRef<str>>(uri: &Url, text: S) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didChange",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                        },
                        "contentChanges": [
                            {
                                "text": text.as_ref(),
                            }
                        ],
                    },
                })
            }
        }
    }

    pub mod did_open {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        pub fn notification<S: AsRef<str>, T: AsRef<str>>(
            uri: &Url,
            language_id: S,
            version: i64,
            text: T,
        ) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": language_id.as_ref(),
                        "version": version,
                        "text": text.as_ref(),
                    },
                },
            })
        }
    }

    pub mod definition {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, position: Position) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/definition",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                },
                "id": 1,
            })
        }
    }

    pub mod hover {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        pub fn request(uri: &Url, position: Position) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                },
                "id": 1,
            })
        }
    }

    pub mod references {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, position: Position, include_declaration: bool) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/references",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                    "context": {
                        "includeDeclaration": include_declaration,
                    },
                },
                "id": 1,
            })
        }
    }

    pub mod highlight {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, position: Position) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/documentHighlight",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                },
                "id": 1,
            })
        }
    }

    pub mod document_symbol {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/documentSymbol",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                },
                "id": 1,
            })
        }
    }

    fn format_options() -> serde_json::Value {
        serde_json::json!({
            "tabSize": 4,
            "insertSpaces": true,
        })
    }

    pub mod formatting {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/formatting",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "options": super::format_options(),
                },
                "id": 1,
            })
        }
    }

    pub mod range_formatting {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, range: &Range) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/rangeFormatting",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "range": range,
                    "options": super::format_options(),
                },
                "id": 1,
            })
        }
    }

    pub mod prepare_rename {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, position: &Position) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/prepareRename",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                },
                "id": 1,
            })
        }
    }

    pub mod rename {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, position: &Position, new_name: &str) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/rename",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": position,
                    "newName": new_name,
                },
                "id": 1,
            })
        }
    }

    pub mod folding_range {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/foldingRange",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                },
                "id": 1,
            })
        }
    }

    pub mod selection_range {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, positions: &[Position]) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/selectionRange",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "positions": positions,
                },
                "id": 1,
            })
        }
    }

    pub mod code_action {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, range: &Range) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/codeAction",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "range": range,
                    "context": {
                        "diagnostics": [],
                    }
                },
                "id": 1,
            })
        }
    }
}

mod workspace {
    pub mod symbol {
        use serde_json::{json, Value};

        #[allow(unused)]
        pub fn request(query: &str) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "workspace/symbol",
                "params": {
                    "query": query,
                },
                "id": 1,
            })
        }
    }
}

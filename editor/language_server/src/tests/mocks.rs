//! Mock service for language server testing.
//! Extended from https://github.com/wasm-lsp/wasm-language-server/blob/main/crates/testing/src/lsp.rs

use serde_json::Value;
use tokio::stream::StreamExt;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, MessageStream};
use tower_test::mock::Spawn;

pub struct MockService {
    service: Spawn<LspService>,
    msg_stream: MessageStream,
}

impl MockService {
    pub async fn default() -> Self {
        Self::new(false).await
    }

    /// Creates a new slide language service and initializes it.
    pub async fn new(link_support: bool) -> Self {
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
                initialize::request(link_support),
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

    pub async fn code_action(
        &mut self,
        uri: &Url,
        range: Range,
        existing_diagnostics: Vec<Diagnostic>,
    ) -> Option<Vec<CodeActionOrCommand>> {
        self.assert_ready();
        let action_resp = self
            .send(text_document::code_action::request(uri, range, existing_diagnostics))
            .await
            .unwrap();
        serde_json::from_value(action_resp.get("result").unwrap().clone()).ok()
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

    pub fn request(link_support: bool) -> Value {
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

    pub mod code_action {
        use serde_json::{json, Value};
        use tower_lsp::lsp_types::*;

        #[allow(unused)]
        pub fn request(uri: &Url, range: Range, existing_diagnostics: Vec<Diagnostic>) -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/codeAction",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "range": range,
                    "context": {
                        "diagnostics": existing_diagnostics,
                    }
                },
                "id": 1,
            })
        }
    }
}

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
    /// Creates a new slide language service and initializes it.
    pub async fn new() -> Self {
        let (service, msg_stream) = LspService::new(crate::SlideLS::new);
        let service = Spawn::new(service);
        let mut service = Self {
            service,
            msg_stream,
        };

        service.assert_ready();

        // Initialize
        service
            .send_recv(initialize::request(), Some(initialize::response()))
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

    pub async fn hover(&mut self, uri: &Url, position: Position) -> Hover {
        self.assert_ready();
        let hover_resp = self
            .send(text_document::hover::request(uri, position))
            .await
            .unwrap();
        serde_json::from_value(hover_resp.get("result").unwrap().clone()).unwrap()
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

    pub fn request() -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "capabilities": {},
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
}

pub fn range_of(subtext: &str, text: &str) -> Range {
    let span_start = text
        .match_indices(subtext)
        .next()
        .expect("Subtext not found.")
        .0;
    let span = (span_start, span_start + subtext.chars().count());
    crate::shims::to_range(&span.into(), text)
}

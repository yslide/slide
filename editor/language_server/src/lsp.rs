//! Crate `slide_ls` implements a language server for [slide](libslide).

#![deny(warnings)]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yslide/slide/base/assets/logo.png")]

use libslide::*;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::ops::Deref;

mod ast;
mod document;
mod init;
mod services;
mod shims;

use document::{ChangeKind, DocumentRegistry, ProgramInfo};
use init::InitializationOptions;

#[cfg(test)]
mod tests;

/// A slide language server.
pub struct SlideLS {
    client: Client,
    // These are always correctly set after `initialize`.
    document_registry: RwLock<Option<DocumentRegistry>>,
    client_caps: RwLock<Option<ClientCapabilities>>,
}

impl SlideLS {
    /// Creates a new language server given a server client.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_registry: RwLock::new(None),
            client_caps: RwLock::new(None),
        }
    }

    /// Returns capabilities of the language server.
    pub fn capabilities() -> ServerCapabilities {
        let text_document_sync = Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::Full),
                ..TextDocumentSyncOptions::default()
            },
        ));
        let definition_provider = Some(true);
        let hover_provider = Some(HoverProviderCapability::Simple(true));
        let references_provider = Some(true);
        let document_highlight_provider = Some(true);

        ServerCapabilities {
            definition_provider,
            text_document_sync,
            hover_provider,
            references_provider,
            document_highlight_provider,
            ..ServerCapabilities::default()
        }
    }

    async fn change(&self, fi: Url, text: String, version: Option<i64>) {
        let diags = self
            .registry_mut()
            .change(ChangeKind::FileModified(fi.clone(), text));
        self.client.publish_diagnostics(fi, diags, version).await;
    }

    fn close(&self, fi: &Url) {
        self.registry_mut()
            .change(ChangeKind::FileRemoved(fi.clone()));
    }

    fn client_caps(&self) -> MappedRwLockReadGuard<ClientCapabilities> {
        RwLockReadGuard::map(self.client_caps.read(), |c| c.as_ref().unwrap())
    }

    fn registry_mut(&self) -> MappedRwLockWriteGuard<DocumentRegistry> {
        RwLockWriteGuard::map(self.document_registry.write(), |r| r.as_mut().unwrap())
    }

    fn registry(&self) -> MappedRwLockReadGuard<DocumentRegistry> {
        RwLockReadGuard::map(self.document_registry.read(), |r| r.as_ref().unwrap())
    }

    fn get_program_info(&self, doc: &Url) -> MappedRwLockReadGuard<ProgramInfo> {
        MappedRwLockReadGuard::map(self.registry(), |r| r.program(doc).unwrap())
    }

    fn context(&self) -> MappedRwLockReadGuard<ProgramContext> {
        MappedRwLockReadGuard::map(self.registry(), |r| r.context())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SlideLS {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let (InitializationOptions { document_parsers }, diags) =
            InitializationOptions::from_json(params.initialization_options);
        for diag in diags {
            self.client
                .log_message(MessageType::Error, diag.to_string())
                .await;
        }

        // TODO: make this a user option
        let context = ProgramContext::default().lint(true);
        let document_registry = DocumentRegistry::new(document_parsers, context);

        // Update fresh instance options
        *self.document_registry.write() = Some(document_registry);
        *self.client_caps.write() = Some(params.capabilities);

        Ok(InitializeResult {
            capabilities: SlideLS::capabilities(),
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "Slide language server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem {
            uri, text, version, ..
        } = params.text_document;
        self.change(uri, text, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let VersionedTextDocumentIdentifier { uri, version, .. } = params.text_document;
        // NOTE: We specify that we expect full-content syncs in the server capabilities,
        // so here we assume the only change passed is a change of the entire document's content.
        let TextDocumentContentChangeEvent { text, .. } =
            params.content_changes.into_iter().next().unwrap();
        self.change(uri, text, version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let TextDocumentIdentifier { uri } = params.text_document;
        self.close(&uri);
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params.text_document_position_params;
        let program_info = self.get_program_info(&uri);
        let supports_link = self
            .client_caps()
            .text_document
            .as_ref()
            .and_then(|td| td.definition)
            .and_then(|def| def.link_support)
            .unwrap_or(false);

        let definitions = services::get_definitions(position, program_info.deref(), supports_link);
        Ok(definitions)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params.text_document_position_params;
        let program_info = self.get_program_info(&uri);
        let context = self.context();

        let hover = services::get_hover_info(position, program_info.deref(), context.deref());
        Ok(hover)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let ReferenceParams {
            text_document_position:
                TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri },
                    position,
                },
            context: ReferenceContext {
                include_declaration,
            },
            ..
        } = params;
        let program_info = self.get_program_info(&uri);

        let references =
            services::get_references(position, include_declaration, program_info.deref());
        Ok(references)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params.text_document_position_params;
        let program_info = self.get_program_info(&uri);

        let highlights = services::get_semantic_highlights(position, program_info.deref());
        Ok(highlights)
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(SlideLS::new);
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}

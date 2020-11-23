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

mod ast;
mod document_registry;
mod init;
mod program;
mod ptr;

use document_registry::{Change, DocumentRegistry};
use init::InitializationOptions;
use program::Program;
use ptr::p;

#[cfg(test)]
mod tests;

/// A slide language server.
pub struct SlideLS {
    /// LSP client the server communicates with.
    client: Client,

    ///////////////////////////////////////////////////////////////////////////////
    ////// The following fields are always correctly set after `initialize`. //////
    ///////////////////////////////////////////////////////////////////////////////
    /// The database of documents known to the server session.
    document_registry: RwLock<Option<DocumentRegistry>>,
    /// The [LSP client's](Self::client) capabilities.
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
        let document_symbol_provider = Some(true);
        let workspace_symbol_provider = Some(true);
        let document_formatting_provider = Some(true);
        let document_range_formatting_provider = Some(true);
        let rename_provider = Some(RenameProviderCapability::Options(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }));

        ServerCapabilities {
            definition_provider,
            text_document_sync,
            hover_provider,
            references_provider,
            document_highlight_provider,
            document_symbol_provider,
            workspace_symbol_provider,
            document_formatting_provider,
            document_range_formatting_provider,
            rename_provider,
            ..ServerCapabilities::default()
        }
    }

    /// Records a document content change.
    async fn change(&self, fi: Url, text: String, version: Option<i64>) {
        self.registry_mut()
            .apply_change(Change::Modified(fi.clone(), text));

        let document_diagnostics = self.registry().document(&fi).map(|d| d.all_diagnostics());
        if let Some(diags) = document_diagnostics {
            self.client.publish_diagnostics(fi, diags, version).await;
        }
    }

    /// Records the closing of a document.
    fn close(&self, fi: &Url) {
        self.registry_mut()
            .apply_change(Change::Removed(fi.clone()));
    }

    /// Retrieves the LSP client's capabilities.
    fn client_capabilities(&self) -> MappedRwLockReadGuard<ClientCapabilities> {
        RwLockReadGuard::map(self.client_caps.read(), |c| c.as_ref().unwrap())
    }

    /// Retrieves a reference to the document registry.
    fn registry(&self) -> MappedRwLockReadGuard<DocumentRegistry> {
        RwLockReadGuard::map(self.document_registry.read(), |r| r.as_ref().unwrap())
    }

    /// Retrieves a mutable reference to the document registry.
    fn registry_mut(&self) -> MappedRwLockWriteGuard<DocumentRegistry> {
        RwLockWriteGuard::map(self.document_registry.write(), |r| r.as_mut().unwrap())
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
        let document_registry = DocumentRegistry::new(document_parsers, p(context));

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
        let supports_link = self
            .client_capabilities()
            .text_document
            .as_ref()
            .and_then(|td| td.definition)
            .and_then(|def| def.link_support)
            .unwrap_or(false);

        let definitions =
            self.registry()
                .with_program_at_uri_and_position(&uri, position, |program, offset| {
                    program.get_definitions(offset, supports_link)
                });

        Ok(definitions)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params.text_document_position_params;

        let hover =
            self.registry()
                .with_program_at_uri_and_position(&uri, position, |program, offset| {
                    program.get_hover_info(offset)
                });

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

        let references =
            self.registry()
                .with_program_at_uri_and_position(&uri, position, |program, offset| {
                    program.get_references(offset, include_declaration)
                });

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

        let highlights =
            self.registry()
                .with_program_at_uri_and_position(&uri, position, |program, offset| {
                    program.get_semantic_highlights(offset)
                });

        Ok(highlights)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            ..
        } = params;

        let symbols = self
            .registry()
            .with_programs_at_uri(&uri, |program| Some(program.get_symbols(None)));
        let symbols = symbols.map(|s| DocumentSymbolResponse::Flat(s.concat()));

        Ok(symbols)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let WorkspaceSymbolParams { query, .. } = params;

        let symbols = self
            .registry()
            .with_all_programs(|program| Some(program.get_symbols(Some(query.as_ref()))));
        let symbols = symbols.map(|s| s.concat());

        Ok(symbols)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            ..
        } = params;

        let formattings = self.registry().with_programs_at_uri(&uri, |program| {
            // TODO: use user emit config
            Some(program.format(EmitConfig::default()))
        });

        Ok(formattings)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let DocumentRangeFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            range,
            ..
        } = params;

        let formatting = self
            .registry()
            .with_program_at_uri_and_range(&uri, range, |program, span| {
                // TODO: use user emit config
                program.format_span(span, EmitConfig::default())
            })
            .map(|f| vec![f]);

        Ok(formatting)
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params;

        let can_rename =
            self.registry()
                .with_program_at_uri_and_position(&uri, position, |program, offset| {
                    Some(program.can_rename(offset))
                });

        can_rename.unwrap_or(Ok(None))
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

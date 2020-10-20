//! Crate `slide_ls` implements a language server for [slide](libslide).

#![deny(warnings)]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yslide/slide/master/assets/logo.png")]

use libslide::*;

use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

mod ast;
mod services;
mod shims;
use shims::convert_diagnostics;

#[cfg(test)]
mod tests;

pub(crate) struct ProgramInfo {
    source: String,
    uri: Url,
    original: StmtList,
    #[allow(unused)]
    simplified: StmtList,
}

type DocumentRegistry = HashMap<Url, ProgramInfo>;

/// A slide language server.
pub struct SlideLS {
    client: Client,
    document_registry: Mutex<RefCell<DocumentRegistry>>,
    // These are always correctly set after `initialize`.
    context: Mutex<RefCell<ProgramContext>>,
    client_caps: Mutex<RefCell<ClientCapabilities>>,
}

impl SlideLS {
    /// Creates a new language server given a server client.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_registry: Default::default(),
            context: Default::default(),
            client_caps: Default::default(),
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
        let completion_provider = Some(CompletionOptions{
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["=", "(", "[", "+", "-", "*", "/", "%", "^"].into_iter().map(String::from).collect()),
            ..CompletionOptions::default()
        });
        let definition_provider = Some(true);
        let hover_provider = Some(HoverProviderCapability::Simple(true));
        let references_provider = Some(true);
        let document_highlight_provider = Some(true);

        ServerCapabilities {
            completion_provider,
            definition_provider,
            text_document_sync,
            hover_provider,
            references_provider,
            document_highlight_provider,
            ..ServerCapabilities::default()
        }
    }

    async fn change(&self, doc: Url, text: String, version: Option<i64>) {
        // On document change, we do the following:
        //   1. Reparse the program source code
        //   2. Evaluate the program
        //      - There is a tradeoff between evaluating everything at once on change and lazily
        //        evaluating on queries. For now, we need to do it in this step because some
        //        diagnostics (i.e. validation) cannot be done without performing evaluation
        //        anyway.
        //        A future flow could be to use a "query" model, whereby we incrementally parse,
        //        evaluate, and publish diagnostics localized to a single statement.
        //        But we are far away from that being important.
        //   3. Since we're already here, publish any diagnostics we discovered.
        //
        // We cache both the original program AST and evaluated AST so we can answer later queries
        // for original/optimized statements without re-evaluation.

        // 1. Parse
        let ScanResult {
            tokens,
            diagnostics: scan_diags,
        } = scan(&*text);
        let ParseResult {
            program,
            diagnostics: parse_diags,
        } = parse_statements(tokens, &text);
        let lint_diags = lint_stmt(&program, &text);
        // 2. Eval
        let EvaluationResult {
            simplified,
            diagnostics: eval_diags,
        } = evaluate(program.clone(), &self.context().deref()).expect("Evaluation failed.");

        // 3. Publish diagnostics
        let diags = [scan_diags, parse_diags, lint_diags, eval_diags]
            .iter()
            .flat_map(|diags| convert_diagnostics(diags, "slide", &doc, &text))
            .collect();
        self.client
            .publish_diagnostics(doc.clone(), diags, version)
            .await;

        // Final: save results
        self.doc_registry().get_mut().insert(
            doc.clone(),
            ProgramInfo {
                source: text,
                uri: doc,
                original: program,
                simplified,
            },
        );
    }

    fn close(&self, doc: &Url) {
        self.doc_registry().get_mut().remove(doc);
    }

    fn doc_registry(&self) -> MutexGuard<RefCell<DocumentRegistry>> {
        self.document_registry.lock()
    }

    fn get_program_info(&self, doc: &Url) -> MappedMutexGuard<ProgramInfo> {
        MutexGuard::map(self.doc_registry(), |dr| dr.get_mut().get_mut(doc).unwrap())
    }

    fn context(&self) -> MappedMutexGuard<ProgramContext> {
        MutexGuard::map(self.context.lock(), |pc| pc.get_mut())
    }

    fn client_caps(&self) -> MappedMutexGuard<ClientCapabilities> {
        MutexGuard::map(self.client_caps.lock(), |pc| pc.get_mut())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SlideLS {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let context = ProgramContext::default().lint(true);
        self.context.lock().replace(context);
        self.client_caps.lock().replace(params.capabilities);

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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position,
        } = params.text_document_position;
        let program_info = self.get_program_info(&uri);

        let completions = services::get_completions(position, program_info.deref());
        Ok(completions)
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

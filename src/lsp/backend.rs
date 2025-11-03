//! LSP backend implementation
//!
//! This module implements the Language Server Protocol interface
//! for the Snek LSP server.

use std::sync::Arc;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document_store::DocumentStore;
use crate::model::ModelClient;
use crate::snapshot::ContextSnapshot;

/// Request parameters for snek/inline custom method
#[derive(Debug, Deserialize)]
pub struct InlineCompletionParams {
    /// Document URI
    pub text_document: TextDocumentIdentifier,
    /// Cursor position
    pub position: Position,
}

/// Response for snek/inline custom method
#[derive(Debug, Serialize)]
pub struct InlineCompletionResponse {
    /// Generated completion text
    pub completion: String,
}

/// LSP backend state
pub struct Backend {
    /// LSP client for sending notifications
    pub client: Client,
    /// Shared snapshot of session state
    pub snapshot: Arc<ArcSwap<ContextSnapshot>>,
    /// Document content tracker
    pub documents: Arc<DocumentStore>,
    /// AI model client
    pub model: Arc<ModelClient>,
}

impl Backend {
    /// Create a new backend
    pub fn new(
        client: Client,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
        documents: Arc<DocumentStore>,
        model: Arc<ModelClient>,
    ) -> Self {
        Self {
            client,
            snapshot,
            documents,
            model,
        }
    }

    /// Handle custom snek/inline completion request
    pub async fn handle_inline_completion(
        &self,
        params: InlineCompletionParams,
    ) -> jsonrpc::Result<InlineCompletionResponse> {
        let uri = params.text_document.uri.to_string();
        let line = params.position.line;
        let character = params.position.character;

        eprintln!(
            "[SNEK] Inline completion request: uri={}, line={}, char={}",
            uri, line, character
        );

        // Get prefix/suffix from document store
        let (prefix, suffix, language) = self
            .documents
            .get_context(&uri, line, character)
            .ok_or_else(|| {
                eprintln!("[SNEK] ERROR: Document not found in store: {}", uri);
                jsonrpc::Error::invalid_params("Document not found or position invalid")
            })?;

        eprintln!(
            "[SNEK] Context retrieved: language={}, prefix_len={}, suffix_len={}",
            language,
            prefix.len(),
            suffix.len()
        );

        // Load current snapshot
        let snapshot = self.snapshot.load();

        // Call AI model
        let completion = self
            .model
            .complete(&snapshot, &prefix, &suffix, &language)
            .await
            .map_err(|e| {
                let error_msg = format!("Model API error: {}", e);
                eprintln!("[SNEK] {}", error_msg);
                jsonrpc::Error {
                    code: jsonrpc::ErrorCode::InternalError,
                    message: error_msg.into(),
                    data: None,
                }
            })?;

        // Trim leading/trailing whitespace to avoid newline issues
        let completion = completion.trim_start().to_string();
        
        eprintln!("[SNEK] Completion generated: {} chars", completion.len());

        Ok(InlineCompletionResponse { completion })
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "snek-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Snek LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.documents
            .did_open(doc.uri.to_string(), doc.language_id, doc.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some(change) = params.content_changes.first() {
            self.documents.did_change(&uri, change.text.clone());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.did_close(&uri);
    }
}

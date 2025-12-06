use std::sync::Arc;
use tokio::sync::RwLock;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document_store::DocumentStore;
use crate::model::ModelClient;
use crate::snapshot::ContextSnapshot;

#[derive(Debug, Deserialize)]
pub struct InlineCompletionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Serialize)]
pub struct InlineCompletionResponse {
    pub completion: String,
}

pub struct Backend {
    pub client: Client,
    pub snapshot: Arc<ArcSwap<ContextSnapshot>>,
    pub documents: Arc<DocumentStore>,
    pub model: Arc<ModelClient>,
    pub api_key: Arc<RwLock<String>>,
}

impl Backend {
    pub fn new(
        client: Client,
        snapshot: Arc<ArcSwap<ContextSnapshot>>,
        documents: Arc<DocumentStore>,
        model: Arc<ModelClient>,
        api_key: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            client,
            snapshot,
            documents,
            model,
            api_key,
        }
    }

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

        let snapshot = self.snapshot.load();
        let api_key = self.api_key.read().await.clone();

        let completion = self
            .model
            .complete(&snapshot, &prefix, &suffix, &language, &uri, &api_key)
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

        let completion = completion.trim_start().to_string();
        
        eprintln!("[SNEK] Completion generated: {} chars", completion.len());

        Ok(InlineCompletionResponse { completion })
    }

    async fn load_configuration(&self) -> Result<(), String> {
        let config_items = vec![
            ConfigurationItem {
                scope_uri: None,
                section: Some("snek.apiKey".to_string()),
            },
            ConfigurationItem {
                scope_uri: None,
                section: Some("snek.model".to_string()),
            },
        ];

        match self.client.configuration(config_items).await {
            Ok(configs) => {
                if let Some(Value::String(api_key)) = configs.get(0) {
                    if !api_key.is_empty() {
                        let mut key = self.api_key.write().await;
                        *key = api_key.clone();
                        eprintln!("[SNEK] API key loaded from VSCode settings");
                        self.client
                            .log_message(MessageType::INFO, "Snek API key configured")
                            .await;
                    } else {
                        eprintln!("[SNEK] API key is empty in VSCode settings");
                        self.client
                            .show_message(
                                MessageType::WARNING,
                                "Snek API key not configured. Please set 'snek.apiKey' in VSCode settings.",
                            )
                            .await;
                    }
                } else {
                    eprintln!("[SNEK] API key not found in VSCode settings");
                    self.client
                        .show_message(
                            MessageType::WARNING,
                            "Snek API key not configured. Please set 'snek.apiKey' in VSCode settings.",
                        )
                        .await;
                }

                if let Some(Value::String(model)) = configs.get(1) {
                    if !model.is_empty() {
                        self.model.set_model_name(model.clone()).await;
                        eprintln!("[SNEK] Model configured: {}", model);
                        self.client
                            .log_message(MessageType::INFO, format!("Snek model set to: {}", model))
                            .await;
                    }
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("[SNEK] Failed to request configuration: {:?}", e);
                Err(format!("Failed to request configuration: {:?}", e))
            }
        }
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

        if let Err(e) = self.load_configuration().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Failed to load configuration: {}", e),
                )
                .await;
        }
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        if let Err(e) = self.load_configuration().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Failed to reload configuration: {}", e),
                )
                .await;
        }
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
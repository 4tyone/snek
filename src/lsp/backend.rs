use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    InitializeParams, InitializeResult, InitializedParams, ServerCapabilities,
    CompletionOptions, CompletionParams, CompletionResponse, CompletionItem,
    CompletionItemKind, InsertTextFormat, Position, Range,
    request::Request,
};
use tower_lsp::{Client, LanguageServer, async_trait};

pub struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

// Custom inline completion types (LSP 3.18+)
#[derive(Debug, Serialize, Deserialize)]
pub struct InlineCompletionParams {
    pub position: Position,
    pub context: InlineCompletionContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineCompletionContext {
    #[serde(rename = "triggerKind")]
    pub trigger_kind: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineCompletionItem {
    #[serde(rename = "insertText")]
    pub insert_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InlineCompletionList {
    pub items: Vec<InlineCompletionItem>,
}

// Define custom request for inline completion
pub struct InlineCompletionRequest;

impl Request for InlineCompletionRequest {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/inlineCompletion";
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let capabilities = ServerCapabilities {
            // Enable standard completions
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                all_commit_characters: None,
                resolve_provider: Some(false),
                work_done_progress_options: Default::default(),
                completion_item: None,
            }),
            ..Default::default()
        };
        
        Ok(InitializeResult {
            capabilities,
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "Snek LSP".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                "Snek LSP initialized - ready for inline completions",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!(
                    "Completion requested at {}:{}",
                    params.text_document_position.position.line,
                    params.text_document_position.position.character
                ),
            )
            .await;

        // Create a dummy completion item that acts as ghost text
        let completion_item = CompletionItem {
            label: "Ghost text suggestion".to_string(),
            label_details: None,
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Snek AI completion".to_string()),
            documentation: None,
            deprecated: Some(false),
            preselect: Some(true),
            sort_text: Some("0".to_string()), // Make it appear first
            filter_text: None,
            insert_text: Some("// This is a dummy ghost text from Snek LSP\nfn example_function() {\n    println!(\"Hello from AI completion!\");\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        };

        Ok(Some(CompletionResponse::Array(vec![completion_item])))
    }
}

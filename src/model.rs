//! AI model integration for code completion
//!
// 111010011011
//! This module handles communication with OpenAI-compatible APIs
//! to generate code completions based on context.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::snapshot::ContextSnapshot;

/// OpenAI API message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Thinking {
    #[serde(rename = "type")]
    thinking_type: String,
}

#[derive(Debug, Serialize)]
struct ExtraBody {
    thinking: Thinking,
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
    extra_body: ExtraBody,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

/// Client for interacting with the AI model
pub struct ModelClient {
    api_url: String,
    api_key: String,
    model_name: String,
    http_client: reqwest::Client,
}

impl ModelClient {
    /// Create a new model client
    pub fn new(api_url: String, api_key: String, model_name: String) -> Self {
        Self {
            api_url,
            api_key,
            model_name,
            http_client: reqwest::Client::new(),
        }
    }

    /// Generate a code completion
    pub async fn complete(
        &self,
        snapshot: &ContextSnapshot,
        prefix: &str,
        suffix: &str,
        language: &str,
    ) -> Result<String> {
        let messages = build_messages(snapshot, prefix, suffix, language);

        let request = OpenAIRequest {
            model: self.model_name.clone(),
            messages,
            temperature: 0.0,
            max_tokens: snapshot.limits.max_tokens,
            stream: false,
            extra_body: ExtraBody {
                thinking: Thinking {
                    thinking_type: "disabled".to_string(),
                },
            },
        };

        eprintln!("[SNEK] Request details:");
        eprintln!("  - Model: {}", self.model_name);
        eprintln!("  - URL: {}", self.api_url);
        eprintln!("  - Max tokens: {}", snapshot.limits.max_tokens);
        
        let response = self
            .http_client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to AI model")?;

        let status = response.status();
        eprintln!("[SNEK] Response status: {}", status);
        
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            eprintln!("[SNEK] Error response body: {}", body);
            anyhow::bail!("AI model request failed: {} - {}", status, body);
        }

        let response_body: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse AI model response")?;

        let completion = response_body
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(completion)
    }
}

/// Build the message array for the AI model
///
/// Includes system prompt, markdown context, code snippets, and current buffer context
fn build_messages(
    snapshot: &ContextSnapshot,
    prefix: &str,
    suffix: &str,
    language: &str,
) -> Vec<OpenAIMessage> {
    let mut messages = vec![];

    // System prompt
    messages.push(OpenAIMessage {
        role: "system".to_string(),
        content: "You are an AI code completion assistant. Generate code that naturally continues from the given prefix. Return ONLY the completion code without explanations, markdown formatting, or code fences.".to_string(),
    });

    // Build the final user message with contexts and current buffer
    let mut context_msg = String::new();

    // Add markdown context if available
    if !snapshot.markdown_context.is_empty() {
        eprintln!("[SNEK] Including markdown context: {} chars", snapshot.markdown_context.len());
        context_msg.push_str("Here is some context you might need:\n\n");
        context_msg.push_str(&snapshot.markdown_context);
        context_msg.push_str("\n\n---\n\n");
    } else {
        eprintln!("[SNEK] No markdown context available");
    }

    // Add code snippets if available
    if !snapshot.code_snippets.is_empty() {
        eprintln!("[SNEK] Including {} code snippets", snapshot.code_snippets.len());
        context_msg.push_str("Here are some code snippets that you will need:\n\n");
        for (idx, snippet) in snapshot.code_snippets.iter().enumerate() {
            context_msg.push_str(&format!(
                "Snippet {}:\n  URI: {}\n  Lines: {}-{}\n  Language: {}\n",
                idx + 1,
                snippet.uri,
                snippet.start_line,
                snippet.end_line,
                snippet.language_id
            ));
            if let Some(ref desc) = snippet.description {
                context_msg.push_str(&format!("  Description: {}\n", desc));
            }
            context_msg.push_str(&format!("  Code:\n```\n{}\n```\n\n", snippet.code));
        }
        context_msg.push_str("---\n\n");
    }

    // Add current buffer context
    context_msg.push_str(&format!(
        "Complete the following {} code. The cursor is at <CURSOR>.\n\n",
        language
    ));
    context_msg.push_str("Code before cursor:\n```\n");
    context_msg.push_str(prefix);
    context_msg.push_str("\n```\n\n<CURSOR>\n\n");

    if !suffix.trim().is_empty() {
        context_msg.push_str("Code after cursor:\n```\n");
        context_msg.push_str(suffix);
        context_msg.push_str("\n```\n\n");
    }

    context_msg.push_str("Generate ONLY the code that should be inserted at <CURSOR>. Do not include any explanations or markdown formatting.");

    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: context_msg,
    });

    messages
}

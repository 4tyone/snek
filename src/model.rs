//! AI model integration for code completion
//!
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
    model_name: String,
    http_client: reqwest::Client,
}

impl ModelClient {
    /// Create a new model client
    pub fn new(api_url: String, model_name: String) -> Self {
        Self {
            api_url,
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
        api_key: &str,
    ) -> Result<String> {
        // Check if API key is configured
        if api_key.is_empty() {
            anyhow::bail!(
                "API key not configured. Please add your API key in VSCode settings:\n\
                File > Preferences > Settings > Search for 'snek.apiKey'"
            );
        }

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
            .header("Authorization", format!("Bearer {}", api_key))
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
        content: "You are an AI code completion assistant.
        Generate code that naturally continues from the given prefix.
        The developer trusts you to understand what they are trying to make, and your continuation needs to be very helpful to not get in the way of the developer.
        IMPORTANT NOTES:
            1. Preserve the indentation level from the context. Match the indentation of surrounding code.
            2. Do not repeat existing code blindly, try to understand what the developer whants to write and continue the thought naturally
        
        Return ONLY the completion code without explanations, markdown formatting (``` ```), or code fences.
        You will be given some context - markdown files that tell you about the developer intent, what are they making and what they want to achieve, or general information about the code base, it can be anything, most importantly the data is in natural language, may contain code snippets etc.
        You will also be given code files that will give you more data about the code base".to_string(),
    });

    // Build the final user message with contexts and current buffer
    let mut context_msg = String::new();

    // Add markdown context from cache if available
    if !snapshot.markdown_cache.is_empty() {
        eprintln!("[SNEK] Including {} markdown files", snapshot.markdown_cache.len());
        context_msg.push_str("Here is some context you might need:\n\n");

        // Sort keys for consistent ordering
        let mut filenames: Vec<&String> = snapshot.markdown_cache.keys().collect();
        filenames.sort();

        for filename in filenames {
            if let Some(content) = snapshot.markdown_cache.get(filename) {
                context_msg.push_str(&format!("## {}\n\n", filename));
                context_msg.push_str(content);
                context_msg.push_str("\n\n---\n\n");
            }
        }
    } else {
        eprintln!("[SNEK] No markdown context available");
    }

    // Add code snippets from cache if available
    if !snapshot.code_snippets.is_empty() {
        eprintln!("[SNEK] Including {} code snippets", snapshot.code_snippets.len());
        context_msg.push_str("Here are some code snippets that you will need:\n\n");
        for (idx, snippet) in snapshot.code_snippets.iter().enumerate() {
            context_msg.push_str(&format!(
                "Snippet {}:\n\n  URI: {}\n\n  Lines: {}-{}\n\n  Language: {}\n\n",
                idx + 1,
                snippet.uri,
                snippet.start_line,
                snippet.end_line,
                snippet.language_id
            ));
            if let Some(ref desc) = snippet.description {
                context_msg.push_str(&format!("  Description: {}\n", desc));
            }

            // Get code from cache and extract line range
            if let Some(full_content) = snapshot.file_cache.get(&snippet.uri) {
                let lines: Vec<&str> = full_content.lines().collect();
                let start = snippet.start_line as usize;
                let end = (snippet.end_line as usize).min(lines.len());

                if start < lines.len() {
                    let extracted_lines = &lines[start..end];
                    let code = extracted_lines.join("\n");
                    context_msg.push_str(&format!("  Code:\n```\n{}\n```\n\n", code));
                } else {
                    eprintln!("[SNEK] Warning: Line range {}-{} exceeds file length {} for {}",
                             start, end, lines.len(), snippet.uri);
                    context_msg.push_str("  Code: [Invalid line range]\n\n");
                }
            } else {
                eprintln!("[SNEK] Warning: File not in cache: {}", snippet.uri);
                context_msg.push_str("  Code: [File not in cache]\n\n");
            }
        }
        context_msg.push_str("---\n\n");
    }

    // Add current buffer context with cursor at exact position
    context_msg.push_str(&format!(
        "Complete the following {} code. The cursor is at <CURSOR>. Generate ONLY the code that should be inserted at <CURSOR>. Do not include any explanations or markdown formatting. IMPORTANT: Ensure proper indentation - match the indentation level of the surrounding code context.\n\n",
        language
    ));

    // Add the code with cursor at the exact position
    context_msg.push_str(prefix);
    context_msg.push_str("<CURSOR>");
    context_msg.push_str(suffix);

    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: context_msg,
    });

    messages
}

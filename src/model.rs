use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::snapshot::ContextSnapshot;

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

pub struct ModelClient {
    api_url: String,
    model_name: tokio::sync::RwLock<String>,
    http_client: reqwest::Client,
}

impl ModelClient {
    pub fn new(api_url: String, model_name: String) -> Self {
        Self {
            api_url,
            model_name: tokio::sync::RwLock::new(model_name),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn set_model_name(&self, model_name: String) {
        let mut name = self.model_name.write().await;
        *name = model_name;
    }

    pub async fn complete(
        &self,
        snapshot: &ContextSnapshot,
        prefix: &str,
        suffix: &str,
        language: &str,
        uri: &str,
        api_key: &str,
    ) -> Result<String> {
        if api_key.is_empty() {
            anyhow::bail!(
                "API key not configured. Please add your API key in VSCode settings:\n\
                File > Preferences > Settings > Search for 'snek.apiKey'"
            );
        }

        let model_name = self.model_name.read().await.clone();

        eprintln!("[SNEK] Request details:");
        eprintln!("  - Model: {}", model_name);
        eprintln!("  - URL: {}", self.api_url);
        eprintln!("  - Max tokens: {}", snapshot.limits.max_tokens);

        let messages = build_messages(snapshot, prefix, suffix, language, uri);

        let request = OpenAIRequest {
            model: model_name.clone(),
            messages,
            temperature: 0.0,
            max_tokens: snapshot.limits.max_tokens,
            stream: false,
        };

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

        let response_text = response.text().await.context("Failed to get response text")?;
        eprintln!("[SNEK] Raw response: {}", &response_text[..response_text.len().min(500)]);

        let response_body: OpenAIResponse = serde_json::from_str(&response_text)
            .context("Failed to parse AI model response")?;

        let raw_completion = response_body
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let completion = extract_code_from_response(&raw_completion);

        eprintln!("[SNEK] Raw completion length: {} chars", raw_completion.len());
        eprintln!("[SNEK] Extracted completion length: {} chars", completion.len());

        Ok(completion)
    }
}

fn build_messages(
    snapshot: &ContextSnapshot,
    prefix: &str,
    suffix: &str,
    language: &str,
    uri: &str,
) -> Vec<OpenAIMessage> {
    let mut messages = vec![];

    messages.push(OpenAIMessage {
        role: "system".to_string(), /// TODO: refine the prompt
        content: "You are an AI code completion assistant.
        Generate code that naturally continues from the <CURSOR> position.
        The developer trusts you to understand what they are trying to make, and your continuation needs to be very helpful to not get in the way of the developer.
        IMPORTANT NOTES:
            1. Preserve the indentation level from the context. Match the indentation of surrounding code.
            2. Do not repeat existing code blindly, try to understand what the developer whants to write and continue the thought naturally
                a. Your response will be inserted at the <CURSOR> location so DO NOT repeat the text that comes after it, just continue naturally. e.g. if it goes something like this `...def fibona<CURSOR>...` your response needs to be `cci(int: n)...`
            3. Keep it short and generate code block-by-block. Avoid generating several functions in one response, close that logical block and stop.
        Return ONLY the completion code without explanations, markdown formatting (```python ```, or ```rust ```, or ```typescript ``` or ANYTHING like that), or code fences.
        You will be given some context - markdown files that tell you about the developer intent, what are they making and what they want to achieve, or general information about the code base, it can be anything, most importantly the data is in natural language, may contain code snippets etc.
        You will also be given code files that will give you more data about the code base".to_string(),
        reasoning_content: None,
    });

    let mut context_msg = String::new();

    if !snapshot.markdown_cache.is_empty() {
        eprintln!("[SNEK] Including {} markdown files", snapshot.markdown_cache.len());
        context_msg.push_str("Here is some context you might need:\n\n");

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

    if !snapshot.code_snippets.is_empty() {
        eprintln!("[SNEK] Including {} code snippets", snapshot.code_snippets.len());
        context_msg.push_str("Here are some code snippets that you might need:\n\n");
        for (idx, snippet) in snapshot.code_snippets.iter().enumerate() {
            context_msg.push_str(&format!(
                "Snippet {}\n\n:\n\n  URI: {}\n\n  Lines: {}-{}\n\n  Language: {}\n\n",
                idx + 1,
                snippet.uri,
                snippet.start_line,
                snippet.end_line,
                snippet.language_id
            ));
            if let Some(ref desc) = snippet.description {
                context_msg.push_str(&format!("  Description: {}\n", desc));
            }

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

    context_msg.push_str(&format!(
        "Complete the following code.\n\n{}\n\n\n The cursor is at <CURSOR>. Generate the raw, full code that should be inserted at <CURSOR>. Do not include any explanations or markdown formatting. IMPORTANT: Ensure proper indentation - match the indentation level of the surrounding code context.\n\n",
        language
    ));

    context_msg.push_str(&format!("File: {}\n\n", uri));

    context_msg.push_str(prefix);
    context_msg.push_str("<CURSOR>");
    context_msg.push_str(suffix);

    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: context_msg,
        reasoning_content: None,
    });

    messages
}

fn extract_code_from_response(response: &str) -> String {
    let trimmed = response.trim();

    if trimmed.starts_with("```") {
        if let Some(first_newline) = trimmed.find('\n') {
            let after_lang = &trimmed[first_newline + 1..];
            if let Some(closing_fence_pos) = after_lang.rfind("```") {
                return after_lang[..closing_fence_pos].trim().to_string();
            }
        }
    }

    trimmed.to_string()
}

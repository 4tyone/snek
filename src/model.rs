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
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
    /// Optional reasoning content (used by some models like Cerebras)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reasoning_content: Option<String>,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Thinking {
//     #[serde(rename = "type")]
//     disable_reasoning: bool,
// }

// #[derive(Debug, Serialize)]
// struct ExtraBody {
//     reasoning_effort: String,
// }

#[derive(Debug, Serialize)]
struct ExtraBody {
    disable_reasoning: bool,
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_body: Option<ExtraBody>,
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

/// Completion API request format (Fill-in-the-Middle)
#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    suffix: Option<String>,
    temperature: f32,
    max_tokens: usize,
    stream: bool,
}

/// DeepSeek Completion API response format
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct CompletionChoice {
    text: String,
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
        uri: &str,
        api_key: &str,
    ) -> Result<String> {
        // Check if API key is configured
        if api_key.is_empty() {
            anyhow::bail!(
                "API key not configured. Please add your API key in VSCode settings:\n\
                File > Preferences > Settings > Search for 'snek.apiKey'"
            );
        }

        eprintln!("[SNEK] Request details:");
        eprintln!("  - Model: {}", self.model_name);
        eprintln!("  - URL: {}", self.api_url);
        eprintln!("  - Max tokens: {}", snapshot.limits.max_tokens);

        // Check if using completions API (by URL path)
        // let is_completions_api = self.api_url.contains("/completions") && !self.api_url.contains("/chat/completions");

        // // Check if this is the qwen model (use FIM format even for chat)
        // let use_fim_format = self.model_name.contains("qwen");

        // if is_completions_api {
        //     // Build prompt for fill-in-the-middle with FIM markers
        //     let prompt = build_fim_prompt(snapshot, prefix, language, uri);

        //     // Wrap suffix with FIM markers
        //     let formatted_suffix = if !suffix.is_empty() {
        //         Some(format!("<|fim_suffix|>\n{}<|fim_middle|>", suffix))
        //     } else {
        //         Some("<|fim_suffix|><|fim_middle|>".to_string())
        //     };

        //     let request = CompletionRequest {
        //         model: self.model_name.clone(),
        //         prompt,
        //         suffix: formatted_suffix,
        //         temperature: 0.0,
        //         max_tokens: snapshot.limits.max_tokens,
        //         stream: false,
        //     };

        //     let response = self
        //         .http_client
        //         .post(&self.api_url)
        //         .header("Authorization", format!("Bearer {}", api_key))
        //         .header("Content-Type", "application/json")
        //         .json(&request)
        //         .send()
        //         .await
        //         .context("Failed to send request to AI model")?;

        //     let status = response.status();
        //     eprintln!("[SNEK] Response status: {}", status);

        //     if !status.is_success() {
        //         let body = response.text().await.unwrap_or_default();
        //         eprintln!("[SNEK] Error response body: {}", body);
        //         anyhow::bail!("AI model request failed: {} - {}", status, body);
        //     }

        //     // Get response as text first for debugging
        //     let response_text = response.text().await.context("Failed to get response text")?;
        //     eprintln!("[SNEK] Raw response: {}", &response_text[..response_text.len().min(500)]);

        //     let response_body: CompletionResponse = serde_json::from_str(&response_text)
        //         .context("Failed to parse AI model response")?;

        //     let completion = response_body
        //         .choices
        //         .first()
        //         .map(|c| c.text.clone())
        //         .unwrap_or_default();

        //     eprintln!("[SNEK] Completion extracted: {} chars", completion.len());
        //     eprintln!("[SNEK] First 200 chars: {}", &completion.chars().take(200).collect::<String>());

        //     Ok(completion)
        // } else {
            // Use chat completions format
            // let messages = if use_fim_format {
            //     build_fim_chat_messages(snapshot, prefix, suffix, language, uri)
            // } else {
               let messages = build_messages(snapshot, prefix, suffix, language, uri);
            // };

            let request = OpenAIRequest {
                model: self.model_name.clone(),
                messages,
                temperature: 0.0,
                max_tokens: snapshot.limits.max_tokens,
                stream: false,
                extra_body: None,
                // extra_body: Some(ExtraBody {
                //     reasoning_effort: "low".to_string(),
                // }),
                // extra_body: Some(ExtraBody {
                //     disable_reasoning: true,
                // }),
                
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

            // Get response as text first for debugging
            let response_text = response.text().await.context("Failed to get response text")?;
            eprintln!("[SNEK] Raw response: {}", &response_text[..response_text.len().min(500)]);

            let response_body: OpenAIResponse = serde_json::from_str(&response_text)
                .context("Failed to parse AI model response")?;

            let raw_completion = response_body
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            // Extract code from markdown if present
            let completion = extract_code_from_response(&raw_completion);

            eprintln!("[SNEK] Raw completion length: {} chars", raw_completion.len());
            eprintln!("[SNEK] Extracted completion length: {} chars", completion.len());

            Ok(completion)
        }
    }
// }

/// Build the message array for the AI model
///
/// Includes system prompt, markdown context, code snippets, and current buffer context
fn build_messages(
    snapshot: &ContextSnapshot,
    prefix: &str,
    suffix: &str,
    language: &str,
    uri: &str,
) -> Vec<OpenAIMessage> {
    let mut messages = vec![];

    // System prompt
    messages.push(OpenAIMessage {
        role: "system".to_string(),
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
        "Complete the following {} code. The cursor is at <CURSOR>. Generate the raw, full code that should be inserted at <CURSOR>. Do not include any explanations or markdown formatting. IMPORTANT: Ensure proper indentation - match the indentation level of the surrounding code context.\n\n",
        language
    ));

    // Add the current file URI
    context_msg.push_str(&format!("File: {}\n\n", uri));

    // Add the code with cursor at the exact position
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

/// Build chat messages with FIM format for qwen model
///
/// System prompt + user message with FIM markers
fn build_fim_chat_messages(
    snapshot: &ContextSnapshot,
    prefix: &str,
    suffix: &str,
    _language: &str,
    uri: &str,
) -> Vec<OpenAIMessage> {
    let mut messages = vec![];

    // System prompt - same as standard build_messages
    messages.push(OpenAIMessage {
        role: "system".to_string(),
        content: "You are an AI code completion assistant.
        Generate code that naturally continues from the given prefix.
        The developer trusts you to understand what they are trying to make, and your continuation needs to be very helpful to not get in the way of the developer.
        IMPORTANT NOTES:
            1. Preserve the indentation level from the context. Match the indentation of surrounding code.
            2. Do not repeat existing code blindly, try to understand what the developer whants to write and continue the thought naturally
        Return ONLY the completion code without explanations, markdown formatting (```python ```, or ```rust ```, or ```typescript ``` or ANYTHING like that), or code fences.
        You will be given some context - markdown files that tell you about the developer intent, what are they making and what they want to achieve, or general information about the code base, it can be anything, most importantly the data is in natural language, may contain code snippets etc.
        You will also be given code files that will give you more data about the code base".to_string(),
        reasoning_content: None,
    });

    let mut user_content = String::new();

    // Start with FIM prefix marker
    user_content.push_str("<|fim_prefix|>");

    // Add markdown context from cache if available
    if !snapshot.markdown_cache.is_empty() {
        eprintln!("[SNEK] Including {} markdown files", snapshot.markdown_cache.len());
        user_content.push_str("Here is the context that may describe user intent and helpful information:\n\n");

        // Sort keys for consistent ordering
        let mut filenames: Vec<&String> = snapshot.markdown_cache.keys().collect();
        filenames.sort();

        for filename in filenames {
            if let Some(content) = snapshot.markdown_cache.get(filename) {
                user_content.push_str(&format!("## {}\n\n", filename));
                user_content.push_str(content);
                user_content.push_str("\n\n---\n\n");
            }
        }
    } else {
        eprintln!("[SNEK] No markdown context available");
    }

    // Add code snippets from cache if available
    if !snapshot.code_snippets.is_empty() {
        eprintln!("[SNEK] Including {} code snippets", snapshot.code_snippets.len());
        user_content.push_str("Here are codes that are relevant:\n\n");
        for (idx, snippet) in snapshot.code_snippets.iter().enumerate() {
            user_content.push_str(&format!(
                "Snippet {}:\n\n  URI: {}\n\n  Lines: {}-{}\n\n  Language: {}\n\n",
                idx + 1,
                snippet.uri,
                snippet.start_line,
                snippet.end_line,
                snippet.language_id
            ));
            if let Some(ref desc) = snippet.description {
                user_content.push_str(&format!("  Description: {}\n", desc));
            }

            // Get code from cache and extract line range
            if let Some(full_content) = snapshot.file_cache.get(&snippet.uri) {
                let lines: Vec<&str> = full_content.lines().collect();
                let start = snippet.start_line as usize;
                let end = (snippet.end_line as usize).min(lines.len());

                if start < lines.len() {
                    let extracted_lines = &lines[start..end];
                    let code = extracted_lines.join("\n");
                    user_content.push_str(&format!("  Code:\n```\n{}\n```\n\n", code));
                } else {
                    eprintln!("[SNEK] Warning: Line range {}-{} exceeds file length {} for {}",
                             start, end, lines.len(), snippet.uri);
                    user_content.push_str("  Code: [Invalid line range]\n\n");
                }
            } else {
                eprintln!("[SNEK] Warning: File not in cache: {}", snippet.uri);
                user_content.push_str("  Code: [File not in cache]\n\n");
            }
        }
        user_content.push_str("---\n\n");
    }

    // Add current file with URI
    user_content.push_str(&format!("File: {}\n\n", uri));

    // Add prefix (code before cursor)
    user_content.push_str(prefix);

    // Add FIM suffix marker and suffix content
    user_content.push_str("<|fim_suffix|>\n");
    user_content.push_str(suffix);

    // Add FIM middle marker
    user_content.push_str("<|fim_middle|>");

    messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: user_content,
        reasoning_content: None,
    });

    messages
}

/// Build prompt for fill-in-the-middle completion (DeepSeek format)
///
/// Returns just the prompt string (context + prefix). Suffix is sent separately.
fn build_fim_prompt(
    snapshot: &ContextSnapshot,
    prefix: &str,
    _language: &str,
    uri: &str,
) -> String {
    let mut prompt = String::new();

    // Add markdown context from cache if available
    if !snapshot.markdown_cache.is_empty() {
        eprintln!("[SNEK] Including {} markdown files", snapshot.markdown_cache.len());
        prompt.push_str("Here is the context that may describe user intent and helpful information:\n\n");

        // Sort keys for consistent ordering
        let mut filenames: Vec<&String> = snapshot.markdown_cache.keys().collect();
        filenames.sort();

        for filename in filenames {
            if let Some(content) = snapshot.markdown_cache.get(filename) {
                prompt.push_str(&format!("## {}\n\n", filename));
                prompt.push_str(content);
                prompt.push_str("\n\n---\n\n");
            }
        }
    } else {
        eprintln!("[SNEK] No markdown context available");
    }

    // Add code snippets from cache if available
    if !snapshot.code_snippets.is_empty() {
        eprintln!("[SNEK] Including {} code snippets", snapshot.code_snippets.len());
        prompt.push_str("Here are codes that are relevant:\n\n");
        for (idx, snippet) in snapshot.code_snippets.iter().enumerate() {
            prompt.push_str(&format!(
                "Snippet {}:\n\n  URI: {}\n\n  Lines: {}-{}\n\n  Language: {}\n\n",
                idx + 1,
                snippet.uri,
                snippet.start_line,
                snippet.end_line,
                snippet.language_id
            ));
            if let Some(ref desc) = snippet.description {
                prompt.push_str(&format!("  Description: {}\n", desc));
            }

            // Get code from cache and extract line range
            if let Some(full_content) = snapshot.file_cache.get(&snippet.uri) {
                let lines: Vec<&str> = full_content.lines().collect();
                let start = snippet.start_line as usize;
                let end = (snippet.end_line as usize).min(lines.len());

                if start < lines.len() {
                    let extracted_lines = &lines[start..end];
                    let code = extracted_lines.join("\n");
                    prompt.push_str(&format!("  Code:\n```\n{}\n```\n\n", code));
                } else {
                    eprintln!("[SNEK] Warning: Line range {}-{} exceeds file length {} for {}",
                             start, end, lines.len(), snippet.uri);
                    prompt.push_str("  Code: [Invalid line range]\n\n");
                }
            } else {
                eprintln!("[SNEK] Warning: File not in cache: {}", snippet.uri);
                prompt.push_str("  Code: [File not in cache]\n\n");
            }
        }
        prompt.push_str("---\n\n");
    }

    // Add the prefix with file URI and FIM markers
    prompt.push_str("The code that needs completion:\n\n");
    prompt.push_str(&format!("File: {}\n\n", uri));
    prompt.push_str("<|fim_prefix|>");
    prompt.push_str(prefix);

    prompt
}

/// Extract raw code from response, removing markdown code fences and explanations
fn extract_code_from_response(response: &str) -> String {
    let trimmed = response.trim();

    // Check if response is wrapped in markdown code fence
    if trimmed.starts_with("```") {
        // Find the first newline after opening fence (language identifier line)
        if let Some(first_newline) = trimmed.find('\n') {
            let after_lang = &trimmed[first_newline + 1..];

            // Find closing fence
            if let Some(closing_fence_pos) = after_lang.rfind("```") {
                // Extract code between fences
                return after_lang[..closing_fence_pos].trim().to_string();
            }
        }
    }

    // Check for common explanation patterns and try to extract just the code
    let lines: Vec<&str> = trimmed.lines().collect();

    // If response starts with explanation text, try to find the actual code
    if !lines.is_empty() {
        let first_line = lines[0].trim().to_lowercase();

        // Skip explanatory first lines like "Here's the code:", "I'll complete this:", etc.
        if first_line.starts_with("here") ||
           first_line.starts_with("i'll") ||
           first_line.starts_with("i can") ||
           first_line.starts_with("the") ||
           first_line.starts_with("this") {
            // Look for code block in the remaining lines
            let rest = lines[1..].join("\n");
            if rest.trim().starts_with("```") {
                // Recursively extract if there's a code block
                return extract_code_from_response(&rest);
            }
        }
    }

    // If no markdown detected, return as-is
    trimmed.to_string()
}

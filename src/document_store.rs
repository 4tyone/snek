//! Document content tracking for the active file
//!
//! This module maintains the content of the currently active document
//! to extract prefix/suffix for completion requests.

use std::sync::RwLock;

/// Internal representation of a document
#[derive(Clone, Debug)]
struct DocumentContent {
    /// Document URI
    uri: String,
    /// Language identifier
    language_id: String,
    /// Full document text
    text: String,
}

/// Stores the currently active document content
#[derive(Default)]
pub struct DocumentStore {
    active_doc: RwLock<Option<DocumentContent>>,
}

impl DocumentStore {
    /// Create a new empty document store
    pub fn new() -> Self {
        Self::default()
    }

    /// Called when a document is opened
    pub fn did_open(&self, uri: String, language_id: String, text: String) {
        let mut doc = self.active_doc.write().unwrap();
        *doc = Some(DocumentContent {
            uri,
            language_id,
            text,
        });
    }

    /// Called when a document is changed (full sync)
    pub fn did_change(&self, uri: &str, text: String) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref mut content) = *doc
            && content.uri == uri {
                content.text = text;
            }
    }

    /// Called when a document is closed
    pub fn did_close(&self, uri: &str) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref content) = *doc
            && content.uri == uri {
                *doc = None;
            }
    }

    /// Get prefix, suffix, and language for a given position
    ///
    /// Returns (prefix, suffix, language_id) where:
    /// - prefix: text from start of file to cursor
    /// - suffix: text from cursor to end of file
    /// - language_id: document language
    pub fn get_context(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Option<(String, String, String)> {
        let doc = self.active_doc.read().unwrap();
        let content = doc.as_ref()?;

        if content.uri != uri {
            return None;
        }

        // Convert line/character to byte offset
        let lines: Vec<&str> = content.text.lines().collect();
        let mut offset = 0;

        for (i, line_text) in lines.iter().enumerate() {
            if i < line as usize {
                offset += line_text.len() + 1; // +1 for newline
            } else if i == line as usize {
                offset += character.min(line_text.len() as u32) as usize;
                break;
            }
        }

        // Ensure offset doesn't exceed text length
        offset = offset.min(content.text.len());

        let prefix = content.text[..offset].to_string();
        let suffix = content.text[offset..].to_string();
        let language_id = content.language_id.clone();

        Some((prefix, suffix, language_id))
    }
}

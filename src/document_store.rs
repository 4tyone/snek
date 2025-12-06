use std::sync::RwLock;

#[derive(Clone, Debug)]
struct DocumentContent {
    uri: String,
    language_id: String,
    text: String,
}

#[derive(Default)]
pub struct DocumentStore {
    active_doc: RwLock<Option<DocumentContent>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn did_open(&self, uri: String, language_id: String, text: String) {
        let mut doc = self.active_doc.write().unwrap();
        *doc = Some(DocumentContent {
            uri,
            language_id,
            text,
        });
    }

    pub fn did_change(&self, uri: &str, text: String) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref mut content) = *doc
            && content.uri == uri {
                content.text = text;
            }
    }

    pub fn did_close(&self, uri: &str) {
        let mut doc = self.active_doc.write().unwrap();
        if let Some(ref content) = *doc
            && content.uri == uri {
                *doc = None;
            }
    }

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

        offset = offset.min(content.text.len());

        let prefix = content.text[..offset].to_string();
        let suffix = content.text[offset..].to_string();
        let language_id = content.language_id.clone();

        Some((prefix, suffix, language_id))
    }
}

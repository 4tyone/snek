//! Integration tests for document_store module

use snek::document_store::DocumentStore;

#[test]
fn test_did_open_and_get_context() {
    let store = DocumentStore::new();

    let uri = "file:///test/file.rs".to_string();
    let text = "fn main() {\n    println!(\"Hello\");\n}".to_string();

    store.did_open(uri.clone(), "rust".to_string(), text.clone());

    // Get context at position (1, 4) - after "    " on line 1
    let result = store.get_context(&uri, 1, 4);
    assert!(result.is_some());

    let (prefix, suffix, lang) = result.unwrap();
    assert_eq!(lang, "rust");
    assert!(prefix.contains("fn main()"));
    assert!(suffix.contains("println!"));
}

#[test]
fn test_did_change() {
    let store = DocumentStore::new();

    let uri = "file:///test/file.rs".to_string();
    let text = "old text".to_string();

    store.did_open(uri.clone(), "rust".to_string(), text);

    // Change the document
    let new_text = "new text".to_string();
    store.did_change(&uri, new_text.clone());

    // Verify the change
    let result = store.get_context(&uri, 0, 3);
    assert!(result.is_some());

    let (prefix, _, _) = result.unwrap();
    assert_eq!(prefix, "new");
}

#[test]
fn test_did_close() {
    let store = DocumentStore::new();

    let uri = "file:///test/file.rs".to_string();
    let text = "some text".to_string();

    store.did_open(uri.clone(), "rust".to_string(), text);

    // Close the document
    store.did_close(&uri);

    // Should return None after close
    let result = store.get_context(&uri, 0, 0);
    assert!(result.is_none());
}

#[test]
fn test_get_context_wrong_uri() {
    let store = DocumentStore::new();

    let uri = "file:///test/file.rs".to_string();
    let text = "some text".to_string();

    store.did_open(uri, "rust".to_string(), text);

    // Try to get context for a different URI
    let result = store.get_context("file:///other/file.rs", 0, 0);
    assert!(result.is_none());
}

#[test]
fn test_prefix_suffix_split() {
    let store = DocumentStore::new();

    let uri = "file:///test/file.rs".to_string();
    let text = "abc\ndef\nghi".to_string();

    store.did_open(uri.clone(), "rust".to_string(), text);

    // Position at start of line 1 (before 'd')
    let result = store.get_context(&uri, 1, 0);
    assert!(result.is_some());

    let (prefix, suffix, _) = result.unwrap();
    assert_eq!(prefix, "abc\n");
    assert_eq!(suffix, "def\nghi");
}

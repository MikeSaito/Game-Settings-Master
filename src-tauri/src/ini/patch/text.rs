use indexmap::IndexMap;
use std::collections::HashMap;

/// Patch the ordered document, preserving comments and duplicate array entries.
pub fn patch_ini_text(
    content: &str,
    updates: &IndexMap<String, IndexMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> String {
    let mut document = crate::ini::document::Document::parse(content);
    document.patch(updates, removals);
    document.text()
}

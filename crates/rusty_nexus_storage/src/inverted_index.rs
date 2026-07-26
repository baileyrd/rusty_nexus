//! Inverted index engine for high-performance full-text search.

use std::collections::{HashMap, HashSet};

/// High-throughput token inverted index.
#[derive(Debug, Clone, Default)]
pub struct InvertedIndex {
    index: HashMap<String, HashSet<String>>,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Index or re-index a file's content.
    pub fn index_document(&mut self, rel_path: &str, content: &str) {
        self.remove_document(rel_path);
        for token in tokenize(content) {
            self.index
                .entry(token)
                .or_insert_with(HashSet::new)
                .insert(rel_path.to_string());
        }
    }

    /// Remove a file from the index.
    pub fn remove_document(&mut self, rel_path: &str) {
        for set in self.index.values_mut() {
            set.remove(rel_path);
        }
    }

    /// Search for documents matching query tokens.
    pub fn search(&self, query: &str) -> HashSet<String> {
        let tokens = tokenize(query);
        if tokens.is_empty() {
            let mut all = HashSet::new();
            for set in self.index.values() {
                all.extend(set.iter().cloned());
            }
            return all;
        }

        let mut result: Option<HashSet<String>> = None;
        for token in tokens {
            let matches = self.index.get(&token).cloned().unwrap_or_default();
            if let Some(res) = result.as_mut() {
                res.retain(|path| matches.contains(path));
            } else {
                result = Some(matches);
            }
        }

        result.unwrap_or_default()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

trait LowercaseExt {
    fn lowercase(&self) -> String;
}

impl LowercaseExt for str {
    fn lowercase(&self) -> String {
        self.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverted_index() {
        let mut idx = InvertedIndex::new();
        idx.index_document("doc1.md", "Sovereign AI Knowledge Base");
        idx.index_document("doc2.md", "Rust performance benchmark");

        let hits = idx.search("Knowledge");
        assert_eq!(hits.len(), 1);
        assert!(hits.contains("doc1.md"));
    }
}

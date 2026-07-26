//! Saved command snippet manager for terminal operations.

use std::collections::HashMap;

/// Saved command snippet descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub name: String,
    pub command: String,
    pub description: String,
}

/// Saved snippet manager.
pub struct SnippetManager {
    snippets: HashMap<String, Snippet>,
}

impl Default for SnippetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SnippetManager {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "rebuild".to_string(),
            Snippet {
                name: "rebuild".to_string(),
                command: "rusty_nexus forge status".to_string(),
                description: "Rebuild index and display forge status".to_string(),
            },
        );
        map.insert(
            "daily".to_string(),
            Snippet {
                name: "daily".to_string(),
                command: "rusty_nexus content daily".to_string(),
                description: "Create or open today's daily note".to_string(),
            },
        );
        map.insert(
            "sync".to_string(),
            Snippet {
                name: "sync".to_string(),
                command: "rusty_nexus remind-me sync".to_string(),
                description: "Synchronize note tasks to RemindMe".to_string(),
            },
        );

        Self { snippets: map }
    }

    pub fn list_snippets(&self) -> Vec<Snippet> {
        let mut list: Vec<Snippet> = self.snippets.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_manager() {
        let manager = SnippetManager::new();
        let list = manager.list_snippets();
        assert_eq!(list.len(), 3);
    }
}

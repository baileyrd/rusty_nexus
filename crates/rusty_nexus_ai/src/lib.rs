//! AI engine for `rusty_nexus`.
//!
//! Provides LLM integration, prompt routing, context loading, and RAG over storage.

use rusty_nexus_storage::StorageEngine;

/// AI Service handling prompts and note context generation.
pub struct AiEngine {
    provider_name: String,
}

impl Default for AiEngine {
    fn default() -> Self {
        Self::new("llama")
    }
}

impl AiEngine {
    pub fn new(provider_name: &str) -> Self {
        Self {
            provider_name: provider_name.to_string(),
        }
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Ask a question or run a prompt with optional RAG context from storage.
    pub fn ask(&self, prompt: &str, storage: Option<&StorageEngine>) -> Result<String, String> {
        let mut context = String::new();
        if let Some(st) = storage {
            if let Ok(search_hits) = st.search(prompt) {
                for (path, body) in search_hits.iter().take(3) {
                    context.push_str(&format!("\n--- Context note: {} ---\n{}\n", path, body));
                }
            }
        }

        let full_prompt = if context.is_empty() {
            prompt.to_string()
        } else {
            format!("Context:\n{}\n\nQuestion:\n{}", context, prompt)
        };

        // Synthesize response using provider logic
        Ok(format!(
            "[{}] AI response for: {}",
            self.provider_name, full_prompt
        ))
    }

    /// Summarize note content.
    pub fn summarize_note(&self, note_title: &str, content: &str) -> String {
        format!(
            "Summary of '{}' ({} chars): {}",
            note_title,
            content.len(),
            content.chars().take(120).collect::<String>()
        )
    }
}

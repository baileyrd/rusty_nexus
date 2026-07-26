//! AI engine for `rusty_nexus`.
//!
//! Provides LLM integration, prompt routing, context loading, vector embeddings, and RAG over storage.

pub mod vector;

use rusty_nexus_storage::StorageEngine;
use vector::{generate_embedding, rank_chunks, VectorChunk};

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

        Ok(format!(
            "[{}] AI response for: {}",
            self.provider_name, full_prompt
        ))
    }

    /// Generate vector embedding for a text string.
    pub fn embed(&self, text: &str) -> Vec<f32> {
        generate_embedding(text)
    }

    /// Perform vector-based RAG query using similarity search over note chunks.
    pub fn vector_rag(&self, query: &str, storage: &StorageEngine) -> Result<String, String> {
        let query_vec = self.embed(query);
        let hits = storage.search("")?; // Fetch notes

        let mut chunks = Vec::new();
        for (i, (path, content)) in hits.iter().enumerate() {
            let vec = generate_embedding(content);
            chunks.push(VectorChunk {
                file_path: path.clone(),
                chunk_index: i,
                text: content.clone(),
                embedding: vec,
            });
        }

        let ranked = rank_chunks(&chunks, &query_vec, 3);
        let mut context = String::new();
        for (score, chunk) in ranked {
            context.push_str(&format!(
                "\n--- Score: {:.2} | Note: {} ---\n{}\n",
                score, chunk.file_path, chunk.text
            ));
        }

        Ok(format!(
            "[{}] Vector RAG Results for '{}':\n{}",
            self.provider_name, query, context
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

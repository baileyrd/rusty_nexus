//! Vector embedding generation and similarity search for AI RAG pipeline.

/// A chunk of text extracted from a note with associated vector embedding.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorChunk {
    pub file_path: String,
    pub chunk_index: usize,
    pub text: String,
    pub embedding: Vec<f32>,
}

/// Compute cosine similarity between two feature vectors.
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (a, b) in v1.iter().zip(v2.iter()) {
        dot_product += a * b;
        norm_a += a * a;
        norm_b += b * b;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }
}

/// Generate a deterministic mock/hash embedding for a text string.
pub fn generate_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; 16];
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        vec[i % 16] += (b as f32) / 255.0;
    }
    // Normalize vector
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}

/// Rank chunks by similarity to a query embedding.
pub fn rank_chunks(chunks: &[VectorChunk], query_embedding: &[f32], top_k: usize) -> Vec<(f32, VectorChunk)> {
    let mut scored: Vec<(f32, VectorChunk)> = chunks
        .iter()
        .map(|chunk| {
            let score = cosine_similarity(&chunk.embedding, query_embedding);
            (score, chunk.clone())
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(top_k).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_similarity() {
        let e1 = generate_embedding("rusty_nexus knowledge base");
        let e2 = generate_embedding("rusty_nexus knowledge base");
        let e3 = generate_embedding("unrelated banana item");

        let sim_same = cosine_similarity(&e1, &e2);
        let sim_diff = cosine_similarity(&e1, &e3);

        assert!((sim_same - 1.0).abs() < 1e-4);
        assert!(sim_diff < sim_same);
    }
}

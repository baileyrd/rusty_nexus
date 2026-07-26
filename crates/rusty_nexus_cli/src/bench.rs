//! Performance benchmark harness for rusty_nexus components.

use std::time::Instant;
use rusty_nexus_ai::vector::generate_embedding;
use rusty_nexus_storage::parser::parse_markdown;
use rusty_nexus_storage::StorageEngine;

/// Result summary of a benchmark run.
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration_ms: f64,
    pub ops_per_sec: f64,
}

/// Run full performance benchmark suite.
pub fn run_benchmarks(storage: &StorageEngine) -> Vec<BenchResult> {
    let mut results = Vec::new();

    // 1. Markdown Parsing Benchmark
    let sample_md = "# Benchmark Note\n- [ ] Task 1\n- [ ] Task 2\nCheck [[target_note]] #benchmark\n"
        .repeat(20);
    let start = Instant::now();
    let iters = 10_000;
    for _ in 0..iters {
        let _ = parse_markdown("bench.md", &sample_md);
    }
    let dur = start.elapsed().as_secs_f64() * 1000.0;
    results.push(BenchResult {
        name: "Markdown Parser".to_string(),
        iterations: iters,
        total_duration_ms: dur,
        ops_per_sec: (iters as f64) / (dur / 1000.0),
    });

    // 2. Vector Embedding Generation Benchmark
    let start_vec = Instant::now();
    let vec_iters = 10_000;
    for _ in 0..vec_iters {
        let _ = generate_embedding("Sovereign AI Knowledge Base Benchmark Text");
    }
    let vec_dur = start_vec.elapsed().as_secs_f64() * 1000.0;
    results.push(BenchResult {
        name: "Vector Embedding Generator".to_string(),
        iterations: vec_iters,
        total_duration_ms: vec_dur,
        ops_per_sec: (vec_iters as f64) / (vec_dur / 1000.0),
    });

    // 3. Search Engine Benchmark
    let start_search = Instant::now();
    let search_iters = 1_000;
    for _ in 0..search_iters {
        let _ = storage.search("benchmark");
    }
    let search_dur = start_search.elapsed().as_secs_f64() * 1000.0;
    results.push(BenchResult {
        name: "Storage Full-Text Search".to_string(),
        iterations: search_iters,
        total_duration_ms: search_dur,
        ops_per_sec: (search_iters as f64) / (search_dur / 1000.0),
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_benchmarks() {
        let temp_dir = std::env::temp_dir().join(format!("bench_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = StorageEngine::init(&temp_dir).expect("init storage");

        let results = run_benchmarks(&storage);
        assert_eq!(results.len(), 3);
        assert!(results[0].ops_per_sec > 0.0);

        let _ = fs::remove_dir_all(temp_dir);
    }
}

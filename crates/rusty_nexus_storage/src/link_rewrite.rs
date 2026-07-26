//! Automatic wikilink rewriter for note renames and refactoring.

use std::fs;
use crate::StorageEngine;

/// Rewrite all occurrences of `[[old_target]]` to `[[new_target]]` across forge notes.
pub fn rewrite_wikilinks(storage: &StorageEngine, old_target: &str, new_target: &str) -> Result<usize, String> {
    let old_stem = extract_stem(old_target);
    let new_stem = extract_stem(new_target);

    if old_stem.is_empty() || new_stem.is_empty() || old_stem == new_stem {
        return Ok(0);
    }

    let hits = storage.search(&old_stem)?;
    let mut updated_files_count = 0;

    let target_pattern_exact = format!("[[{}]]", old_stem);
    let replacement_exact = format!("[[{}]]", new_stem);

    let target_pipe_prefix = format!("[[{}|", old_stem);
    let replacement_pipe_prefix = format!("[[{}|", new_stem);

    for (rel_path, content) in hits {
        if content.contains(&target_pattern_exact) || content.contains(&target_pipe_prefix) {
            let new_content = content
                .replace(&target_pattern_exact, &replacement_exact)
                .replace(&target_pipe_prefix, &replacement_pipe_prefix);

            if new_content != content {
                let abs_path = storage.root_path().join(&rel_path);
                fs::write(abs_path, new_content).map_err(|e| e.to_string())?;
                updated_files_count += 1;
            }
        }
    }

    let _ = storage.rebuild_index();
    Ok(updated_files_count)
}

fn extract_stem(path: &str) -> String {
    let p = std::path::Path::new(path);
    p.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_rewriting() {
        let temp_dir = std::env::temp_dir().join(format!("rewrite_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = StorageEngine::init(&temp_dir).expect("init storage");

        let _ = storage.create_file("alpha.md", "# Alpha").unwrap();
        let _ = storage.create_file("referrer.md", "Check [[alpha]] and [[alpha|display]]").unwrap();

        let count = rewrite_wikilinks(&storage, "alpha", "alpha_renamed").expect("rewrite");
        assert_eq!(count, 1);

        let updated = storage.read_file("referrer.md").unwrap();
        assert!(updated.contains("[[alpha_renamed]]"));
        assert!(updated.contains("[[alpha_renamed|display]]"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

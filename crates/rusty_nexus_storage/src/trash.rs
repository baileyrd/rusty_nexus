//! Trash bin and restorable deletion manager (.forge/trash/).

use std::fs;
use std::path::{Path, PathBuf};

/// Soft-delete a file by moving it into `.forge/trash/`.
pub fn move_to_trash(forge_root: &Path, rel_path: &str) -> Result<PathBuf, String> {
    let abs_src = forge_root.join(rel_path);
    if !abs_src.exists() {
        return Err(format!("File '{}' does not exist", rel_path));
    }

    let trash_dir = forge_root.join(".forge").join("trash");
    let abs_dest = trash_dir.join(rel_path);

    if let Some(parent) = abs_dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::rename(&abs_src, &abs_dest).map_err(|e| e.to_string())?;
    Ok(abs_dest)
}

/// List all trashed files.
pub fn list_trash(forge_root: &Path) -> Result<Vec<String>, String> {
    let trash_dir = forge_root.join(".forge").join("trash");
    if !trash_dir.exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    collect_trash_items(&trash_dir, &trash_dir, &mut list);
    Ok(list)
}

fn collect_trash_items(base: &Path, current: &Path, list: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_trash_items(base, &path, list);
            } else if let Ok(rel) = path.strip_prefix(base) {
                list.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Restore a trashed file back to its original vault position.
pub fn restore_from_trash(forge_root: &Path, rel_path: &str) -> Result<(), String> {
    let trash_dir = forge_root.join(".forge").join("trash");
    let abs_trashed = trash_dir.join(rel_path);
    let abs_dest = forge_root.join(rel_path);

    if !abs_trashed.exists() {
        return Err(format!("Trashed file '{}' not found", rel_path));
    }

    if let Some(parent) = abs_dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::rename(&abs_trashed, &abs_dest).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trash_and_restore() {
        let temp_dir = std::env::temp_dir().join(format!("trash_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        fs::create_dir_all(&temp_dir).unwrap();

        let file_path = temp_dir.join("sample.md");
        fs::write(&file_path, "Sample text").unwrap();

        let _ = move_to_trash(&temp_dir, "sample.md").unwrap();
        assert!(!file_path.exists());

        let trashed = list_trash(&temp_dir).unwrap();
        assert_eq!(trashed, vec!["sample.md"]);

        restore_from_trash(&temp_dir, "sample.md").unwrap();
        assert!(file_path.exists());

        let _ = fs::remove_dir_all(temp_dir);
    }
}

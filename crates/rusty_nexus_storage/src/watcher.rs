//! Asynchronous File Watcher monitoring forge changes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Storage change event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageEvent {
    FileCreated(String),
    FileModified(String),
    FileDeleted(String),
}

/// File watcher tracking forge file system updates.
pub struct Watcher {
    forge_root: PathBuf,
    file_stamps: HashMap<String, SystemTime>,
}

impl Watcher {
    pub fn new(forge_root: &Path) -> Self {
        let mut watcher = Self {
            forge_root: forge_root.to_path_buf(),
            file_stamps: HashMap::new(),
        };
        watcher.scan_initial_state();
        watcher
    }

    fn scan_initial_state(&mut self) {
        let mut current = HashMap::new();
        self.collect_stamps(&self.forge_root.clone(), &mut current);
        self.file_stamps = current;
    }

    fn collect_stamps(&self, dir: &Path, map: &mut HashMap<String, SystemTime>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with('.')) {
                    continue;
                }
                if path.is_dir() {
                    self.collect_stamps(&path, map);
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(rel) = path.strip_prefix(&self.forge_root) {
                        let rel_str = rel.to_string_lossy().replace('\\', "/");
                        if let Ok(meta) = fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                map.insert(rel_str, modified);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check for file system updates since last scan and return events.
    pub fn poll_events(&mut self) -> Vec<StorageEvent> {
        let mut new_stamps = HashMap::new();
        self.collect_stamps(&self.forge_root.clone(), &mut new_stamps);

        let mut events = Vec::new();

        // Check for created or modified files
        for (path, mtime) in &new_stamps {
            if let Some(old_mtime) = self.file_stamps.get(path) {
                if mtime > old_mtime {
                    events.push(StorageEvent::FileModified(path.clone()));
                }
            } else {
                events.push(StorageEvent::FileCreated(path.clone()));
            }
        }

        // Check for deleted files
        for path in self.file_stamps.keys() {
            if !new_stamps.contains_key(path) {
                events.push(StorageEvent::FileDeleted(path.clone()));
            }
        }

        self.file_stamps = new_stamps;
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_detection() {
        let temp_dir = std::env::temp_dir().join(format!("watcher_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        fs::create_dir_all(&temp_dir).unwrap();

        let mut watcher = Watcher::new(&temp_dir);

        // Create new file
        let note_path = temp_dir.join("test.md");
        fs::write(&note_path, "# Test Note").unwrap();

        let events = watcher.poll_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], StorageEvent::FileCreated("test.md".to_string()));

        // Delete file
        fs::remove_file(&note_path).unwrap();
        let events_del = watcher.poll_events();
        assert_eq!(events_del.len(), 1);
        assert_eq!(events_del[0], StorageEvent::FileDeleted("test.md".to_string()));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

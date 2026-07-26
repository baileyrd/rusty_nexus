//! Nexus Storage Engine: Forge management, file reading/writing, markdown indexing,
//! SQLite metadata database via `rusty-db`, search via `rusty-search`, and Knowledge Graph.

pub mod bases;
pub mod canvas;
pub mod export;
pub mod frontmatter;
pub mod git;
pub mod graph;
pub mod link_rewrite;
pub mod parser;
pub mod trash;
pub mod watcher;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use canvas::parse_canvas;
use graph::{GraphStats, KnowledgeGraph};
use parser::parse_markdown;
use rusty_nexus_types::{ForgeMetadata, NoteRecord, TaskRecord};

/// Main Storage Engine managing a Nexus Forge directory.
pub struct StorageEngine {
    root_path: PathBuf,
    _forge_dir: PathBuf,
    graph: Arc<RwLock<KnowledgeGraph>>,
}

impl StorageEngine {
    /// Initialize a new forge at `root_path`.
    pub fn init(root_path: &Path) -> Result<Self, String> {
        if !root_path.exists() {
            fs::create_dir_all(root_path)
                .map_err(|e| format!("Failed to create forge directory: {}", e))?;
        }
        let forge_dir = root_path.join(".forge");
        if !forge_dir.exists() {
            fs::create_dir_all(&forge_dir)
                .map_err(|e| format!("Failed to create .forge directory: {}", e))?;
        }

        let engine = Self {
            root_path: root_path.to_path_buf(),
            _forge_dir: forge_dir,
            graph: Arc::new(RwLock::new(KnowledgeGraph::new())),
        };

        engine.rebuild_index()?;
        Ok(engine)
    }

    /// Open an existing forge at `root_path`.
    pub fn open(root_path: &Path) -> Result<Self, String> {
        let forge_dir = root_path.join(".forge");
        if !forge_dir.exists() {
            return Err(format!(
                "Not a valid Nexus forge: .forge missing in {}",
                root_path.display()
            ));
        }

        let engine = Self {
            root_path: root_path.to_path_buf(),
            _forge_dir: forge_dir,
            graph: Arc::new(RwLock::new(KnowledgeGraph::new())),
        };

        engine.rebuild_index()?;
        Ok(engine)
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Rebuild index by recursively scanning markdown and canvas files in the forge.
    pub fn rebuild_index(&self) -> Result<(), String> {
        let mut g = self.graph.write().unwrap();
        *g = KnowledgeGraph::new();

        self.scan_dir(&self.root_path, &mut g)?;
        Ok(())
    }

    fn scan_dir(&self, dir: &Path, g: &mut KnowledgeGraph) -> Result<(), String> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with('.')) {
                continue; // Skip hidden dirs like .forge, .git
            }

            if path.is_dir() {
                self.scan_dir(&path, g)?;
            } else {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "md" {
                    if let Ok(rel_path) = path.strip_prefix(&self.root_path) {
                        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                        if let Ok(content) = fs::read_to_string(&path) {
                            let parsed = parse_markdown(&rel_str, &content);
                            g.add_node(rel_str.clone(), parsed.title, "note".to_string());
                            for link in parsed.links {
                                g.add_edge(rel_str.clone(), link.target_path);
                            }
                        }
                    }
                } else if ext == "canvas" {
                    if let Ok(rel_path) = path.strip_prefix(&self.root_path) {
                        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(canvas) = parse_canvas(&content) {
                                g.add_node(rel_str.clone(), rel_str.clone(), "canvas".to_string());
                                for target in canvas.extract_file_links() {
                                    g.add_edge(rel_str.clone(), target);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Write content to a note file (vault-relative path).
    pub fn create_file(&self, rel_path: &str, content: &str) -> Result<NoteRecord, String> {
        let abs_path = self.root_path.join(rel_path);
        if let Some(parent) = abs_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::write(&abs_path, content).map_err(|e| e.to_string())?;

        let rel_str = rel_path.replace('\\', "/");
        let parsed = parse_markdown(&rel_str, content);

        {
            let mut g = self.graph.write().unwrap();
            g.remove_node(&rel_str);
            g.add_node(rel_str.clone(), parsed.title.clone(), "note".to_string());
            for link in &parsed.links {
                g.add_edge(rel_str.clone(), link.target_path.clone());
            }
        }

        let record = NoteRecord {
            rel_path: rel_str,
            title: parsed.title,
            content_hash: format!("{:x}", content.len()),
            size_bytes: content.len() as u64,
            created_ms: 0,
            modified_ms: 0,
            tags: parsed.tags,
            links: parsed.links,
            tasks: parsed.tasks,
        };

        Ok(record)
    }

    /// Read raw note content from vault-relative path.
    pub fn read_file(&self, rel_path: &str) -> Result<String, String> {
        let abs_path = self.root_path.join(rel_path);
        fs::read_to_string(&abs_path).map_err(|e| format!("Could not read {}: {}", rel_path, e))
    }

    /// Delete note file.
    pub fn delete_file(&self, rel_path: &str) -> Result<(), String> {
        let abs_path = self.root_path.join(rel_path);
        if abs_path.exists() {
            fs::remove_file(&abs_path).map_err(|e| e.to_string())?;
        }
        let rel_str = rel_path.replace('\\', "/");
        let mut g = self.graph.write().unwrap();
        g.remove_node(&rel_str);
        Ok(())
    }

    /// Perform full-text search across all markdown files.
    pub fn search(&self, query: &str) -> Result<Vec<(String, String)>, String> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        let mut files = Vec::new();
        self.collect_md_files(&self.root_path, &mut files);

        for path in files {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.to_lowercase().contains(&query_lower) {
                    if let Ok(rel) = path.strip_prefix(&self.root_path) {
                        let rel_str = rel.to_string_lossy().replace('\\', "/");
                        results.push((rel_str, content));
                    }
                }
            }
        }

        Ok(results)
    }

    fn collect_md_files(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with('.')) {
                    continue;
                }
                if path.is_dir() {
                    self.collect_md_files(&path, files);
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    files.push(path);
                }
            }
        }
    }

    /// Retrieve all task items across the forge.
    pub fn tasks(&self) -> Result<Vec<TaskRecord>, String> {
        let mut tasks = Vec::new();
        let mut files = Vec::new();
        self.collect_md_files(&self.root_path, &mut files);

        for path in files {
            if let Ok(rel) = path.strip_prefix(&self.root_path) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                if let Ok(content) = fs::read_to_string(&path) {
                    let parsed = parse_markdown(&rel_str, &content);
                    tasks.extend(parsed.tasks);
                }
            }
        }
        Ok(tasks)
    }

    /// Toggle task status by line number.
    pub fn toggle_task(&self, rel_path: &str, line_number: usize) -> Result<bool, String> {
        let abs_path = self.root_path.join(rel_path);
        let content = fs::read_to_string(&abs_path).map_err(|e| e.to_string())?;

        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        if line_number == 0 || line_number > lines.len() {
            return Err("Invalid line number".to_string());
        }

        let line = &mut lines[line_number - 1];
        let mut now_completed = false;
        if line.contains("- [ ]") {
            *line = line.replace("- [ ]", "- [x]");
            now_completed = true;
        } else if line.contains("- [x]") {
            *line = line.replace("- [x]", "- [ ]");
        } else if line.contains("- [X]") {
            *line = line.replace("- [X]", "- [ ]");
        }

        let new_content = lines.join("\n");
        fs::write(&abs_path, new_content).map_err(|e| e.to_string())?;
        Ok(now_completed)
    }

    /// Get backlinks for a file.
    pub fn backlinks(&self, rel_path: &str) -> Vec<String> {
        let rel_str = rel_path.replace('\\', "/");
        let g = self.graph.read().unwrap();
        g.backlinks(&rel_str)
    }

    /// Get overall knowledge graph statistics.
    pub fn graph_stats(&self) -> GraphStats {
        let g = self.graph.read().unwrap();
        g.stats()
    }

    /// Get unresolved links.
    pub fn unresolved_links(&self) -> Vec<String> {
        let g = self.graph.read().unwrap();
        g.unresolved_links()
    }

    /// Get graph status metadata.
    pub fn forge_metadata(&self) -> ForgeMetadata {
        let stats = self.graph_stats();
        ForgeMetadata {
            root_path: self.root_path.to_string_lossy().to_string(),
            created_at_ms: 0,
            note_count: stats.node_count,
            task_count: self.tasks().map(|t| t.len()).unwrap_or(0),
            tag_count: 0,
            version: "0.1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_engine_workflow() {
        let temp_dir = std::env::temp_dir().join(format!("nexus_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = StorageEngine::init(&temp_dir).expect("init storage");

        // 1. Create notes
        let note1 = storage.create_file("notes/alpha.md", "# Alpha\n- [ ] Task 1\nLink to [[beta]] #project").unwrap();
        let _note2 = storage.create_file("beta.md", "# Beta\n- [x] Done task\nLink to [[notes/alpha.md]]").unwrap();

        assert_eq!(note1.rel_path, "notes/alpha.md");

        // 2. Tasks
        let tasks = storage.tasks().unwrap();
        assert_eq!(tasks.len(), 2);

        // 3. Search
        let hits = storage.search("project").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "notes/alpha.md");

        // 4. Backlinks
        let backlinks = storage.backlinks("notes/alpha.md");
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0], "beta.md");

        // 5. Unresolved links
        let unresolved = storage.unresolved_links();
        assert!(unresolved.contains(&"beta".to_string()) || unresolved.is_empty());

        let _ = fs::remove_dir_all(temp_dir);
    }
}

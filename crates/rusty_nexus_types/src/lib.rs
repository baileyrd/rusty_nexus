//! Shared data types for `rusty_nexus`.

use std::fmt;

/// Metadata representing a Nexus Forge directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgeMetadata {
    pub root_path: String,
    pub created_at_ms: u64,
    pub note_count: usize,
    pub task_count: usize,
    pub tag_count: usize,
    pub version: String,
}

impl Default for ForgeMetadata {
    fn default() -> Self {
        Self {
            root_path: String::new(),
            created_at_ms: 0,
            note_count: 0,
            task_count: 0,
            tag_count: 0,
            version: "0.1.0".to_string(),
        }
    }
}

/// A parsed note entry in the storage index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRecord {
    pub rel_path: String,
    pub title: String,
    pub content_hash: String,
    pub size_bytes: u64,
    pub created_ms: u64,
    pub modified_ms: u64,
    pub tags: Vec<String>,
    pub links: Vec<LinkRecord>,
    pub tasks: Vec<TaskRecord>,
}

/// A task item extracted from markdown (`- [ ]` or `- [x]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub id: String,
    pub file_path: String,
    pub line_number: usize,
    pub text: String,
    pub completed: bool,
}

/// A link between notes (`[[wikilink]]` or markdown link).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRecord {
    pub source_path: String,
    pub target_path: String,
    pub link_text: String,
    pub link_type: String,
}

/// A tag associated with a file (`#tag`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRecord {
    pub file_path: String,
    pub tag_name: String,
}

/// A node in the Nexus Knowledge Graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode {
    pub path: String,
    pub title: String,
    pub node_type: String,
}

/// An edge connecting nodes in the Knowledge Graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
}

/// Configuration settings for Nexus.
#[derive(Debug, Clone)]
pub struct NexusConfig {
    pub forge_path: String,
    pub output_format: OutputFormat,
    pub ai_provider: String,
    pub theme: String,
}

impl Default for NexusConfig {
    fn default() -> Self {
        Self {
            forge_path: String::new(),
            output_format: OutputFormat::Text,
            ai_provider: "llama".to_string(),
            theme: "dark".to_string(),
        }
    }
}

/// Supported CLI output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Jsonl,
    Table,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Jsonl => write!(f, "jsonl"),
            Self::Table => write!(f, "table"),
        }
    }
}

/// An IPC message passed through the microkernel event bus.
#[derive(Debug, Clone)]
pub struct IpcMessage {
    pub event_name: String,
    pub payload: String,
}

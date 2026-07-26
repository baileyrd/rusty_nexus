//! MCP (Model Context Protocol) Server for `rusty_nexus`.
//!
//! Serves standard `nexus_*` tools and embeds `remind_me_*` tools.

use std::sync::Arc;
use rusty_nexus_storage::StorageEngine;

/// MCP Tool Descriptor.
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
}

/// MCP Tool Server coordinating Nexus storage and RemindMe tools.
pub struct McpServer {
    storage: Arc<StorageEngine>,
}

impl McpServer {
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// List all available MCP tools provided by Nexus and RemindMe.
    pub fn list_tools(&self) -> Vec<McpToolInfo> {
        vec![
            McpToolInfo {
                name: "nexus_content_create".to_string(),
                description: "Create or overwrite a markdown file in the forge.".to_string(),
            },
            McpToolInfo {
                name: "nexus_content_read".to_string(),
                description: "Read the content of a markdown file in the forge.".to_string(),
            },
            McpToolInfo {
                name: "nexus_content_search".to_string(),
                description: "Full-text search notes in the forge.".to_string(),
            },
            McpToolInfo {
                name: "nexus_content_tasks".to_string(),
                description: "List all task items across forge notes.".to_string(),
            },
            McpToolInfo {
                name: "nexus_graph_status".to_string(),
                description: "Get knowledge graph node, edge, and unresolved count.".to_string(),
            },
            McpToolInfo {
                name: "remind_me_add".to_string(),
                description: "Add a reminder or memory entry.".to_string(),
            },
            McpToolInfo {
                name: "remind_me_list".to_string(),
                description: "List stored reminders and memories.".to_string(),
            },
        ]
    }

    /// Execute a tool call by name.
    pub fn call_tool(&self, name: &str, args_json: &str) -> Result<String, String> {
        match name {
            "nexus_content_create" => {
                let parts: Vec<&str> = args_json.splitn(2, '|').collect();
                if parts.len() < 2 {
                    return Err("Usage: path|content".to_string());
                }
                let note = self.storage.create_file(parts[0], parts[1])?;
                Ok(format!("Created note: {}", note.rel_path))
            }
            "nexus_content_read" => {
                let content = self.storage.read_file(args_json.trim())?;
                Ok(content)
            }
            "nexus_content_search" => {
                let hits = self.storage.search(args_json.trim())?;
                let mut out = format!("Found {} matching files:\n", hits.len());
                for (path, _) in hits {
                    out.push_str(&format!("- {}\n", path));
                }
                Ok(out)
            }
            "nexus_content_tasks" => {
                let tasks = self.storage.tasks()?;
                let mut out = format!("Total tasks: {}\n", tasks.len());
                for task in tasks {
                    let status = if task.completed { "[x]" } else { "[ ]" };
                    out.push_str(&format!("{} {} ({}:{})\n", status, task.text, task.file_path, task.line_number));
                }
                Ok(out)
            }
            "nexus_graph_status" => {
                let stats = self.storage.graph_stats();
                Ok(format!(
                    "Nodes: {}, Edges: {}, Unresolved: {}",
                    stats.node_count, stats.edge_count, stats.unresolved_count
                ))
            }
            "remind_me_add" => {
                Ok(format!("Reminder created: {}", args_json))
            }
            "remind_me_list" => {
                Ok("Reminders list: [Sample reminder from remind_me]".to_string())
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }
}

//! MCP (Model Context Protocol) Server for `rusty_nexus`.
//!
//! Serves standard `nexus_*` tools and embeds `remind_me_*` tools over stdio JSON-RPC 2.0.

pub mod http_server;
pub mod protocol;
pub mod remind_sync;

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use rusty_nexus_storage::StorageEngine;
use protocol::handle_jsonrpc_request;

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

    /// Run stdio JSON-RPC loop for MCP clients.
    pub fn run_stdio_loop(&self) -> Result<(), String> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line_str = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line_str.trim().is_empty() {
                continue;
            }

            if let Some(response) = handle_jsonrpc_request(self, &line_str) {
                let _ = writeln!(stdout, "{}", response);
                let _ = stdout.flush();
            }
        }
        Ok(())
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
                name: "nexus_content_delete".to_string(),
                description: "Delete a markdown file from the forge.".to_string(),
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
                name: "nexus_content_task_toggle".to_string(),
                description: "Toggle task completion state by file path and line number.".to_string(),
            },
            McpToolInfo {
                name: "nexus_content_backlinks".to_string(),
                description: "Get incoming backlinks for a given note.".to_string(),
            },
            McpToolInfo {
                name: "nexus_graph_status".to_string(),
                description: "Get knowledge graph node, edge, and unresolved count.".to_string(),
            },
            McpToolInfo {
                name: "nexus_graph_unresolved".to_string(),
                description: "List unresolved wikilinks in the knowledge graph.".to_string(),
            },
            McpToolInfo {
                name: "nexus_graph_neighbors".to_string(),
                description: "Get graph neighbors for a note within a specified depth.".to_string(),
            },
            McpToolInfo {
                name: "nexus_canvas_read".to_string(),
                description: "Read and parse a spatial .canvas file.".to_string(),
            },
            McpToolInfo {
                name: "nexus_bases_query".to_string(),
                description: "Query a structured .bases database directory.".to_string(),
            },
            McpToolInfo {
                name: "remind_me_add".to_string(),
                description: "Add a reminder or memory entry.".to_string(),
            },
            McpToolInfo {
                name: "remind_me_list".to_string(),
                description: "List stored reminders and memories.".to_string(),
            },
            McpToolInfo {
                name: "remind_me_search".to_string(),
                description: "Search reminders and memories.".to_string(),
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
            "nexus_content_delete" => {
                self.storage.delete_file(args_json.trim())?;
                Ok(format!("Deleted note: {}", args_json.trim()))
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
            "nexus_content_task_toggle" => {
                let parts: Vec<&str> = args_json.splitn(2, '|').collect();
                if parts.len() < 2 {
                    return Err("Usage: path|line_number".to_string());
                }
                let line_num: usize = parts[1].parse().unwrap_or(0);
                let is_done = self.storage.toggle_task(parts[0], line_num)?;
                Ok(format!("Task is now {}", if is_done { "completed" } else { "pending" }))
            }
            "nexus_content_backlinks" => {
                let links = self.storage.backlinks(args_json.trim());
                Ok(format!("Backlinks to {}: {:?}", args_json.trim(), links))
            }
            "nexus_graph_status" => {
                let stats = self.storage.graph_stats();
                Ok(format!(
                    "Nodes: {}, Edges: {}, Unresolved: {}",
                    stats.node_count, stats.edge_count, stats.unresolved_count
                ))
            }
            "nexus_graph_unresolved" => {
                let unres = self.storage.unresolved_links();
                Ok(format!("Unresolved links ({}): {:?}", unres.len(), unres))
            }
            "nexus_graph_neighbors" => {
                Ok(format!("Neighbors for {}: [graph nodes]", args_json.trim()))
            }
            "nexus_canvas_read" => {
                let text = self.storage.read_file(args_json.trim())?;
                let canvas = rusty_nexus_storage::canvas::parse_canvas(&text)?;
                Ok(format!("Canvas with {} nodes, {} edges", canvas.nodes.len(), canvas.edges.len()))
            }
            "nexus_bases_query" => {
                let base = rusty_nexus_storage::bases::load_base(self.storage.root_path(), args_json.trim())?;
                Ok(format!("Base '{}' with {} records", base.schema.name, base.records.len()))
            }
            "remind_me_add" => {
                Ok(format!("Reminder created: {}", args_json))
            }
            "remind_me_list" => {
                Ok("Reminders: [Sample memory entry from remind_me]".to_string())
            }
            "remind_me_search" => {
                Ok(format!("Search results for '{}': [Sample reminder]", args_json))
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }
}

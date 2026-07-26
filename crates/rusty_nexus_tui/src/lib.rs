//! Terminal UI Engine for `rusty_nexus`.
//!
//! Renders interactive dashboard using `rusty_term`, `rusty_lines`, and `rusty_ansi`.

pub mod theme;

use std::sync::Arc;
use rusty_nexus_storage::StorageEngine;

/// Terminal User Interface runner.
pub struct TuiApp {
    storage: Arc<StorageEngine>,
}

impl TuiApp {
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// Launch interactive TUI session.
    pub fn run(&self) -> Result<(), String> {
        println!("\x1b[2J\x1b[H"); // Clear screen & move to top
        println!("\x1b[1;34m====================================================\x1b[0m");
        println!("\x1b[1;36m                 NEXUS TERMINAL UI                  \x1b[0m");
        println!("\x1b[1;34m====================================================\x1b[0m");

        let meta = self.storage.forge_metadata();
        let stats = self.storage.graph_stats();

        println!("\x1b[33mForge Root:\x1b[0m {}", meta.root_path);
        println!("\x1b[33mIndexed Notes:\x1b[0m {}", stats.node_count);
        println!("\x1b[33mKnowledge Graph Edges:\x1b[0m {}", stats.edge_count);
        println!("\x1b[33mUnresolved Links:\x1b[0m {}", stats.unresolved_count);
        println!("\x1b[33mTotal Tasks:\x1b[0m {}", meta.task_count);
        println!("\x1b[1;34m----------------------------------------------------\x1b[0m");

        if let Ok(tasks) = self.storage.tasks() {
            println!("\x1b[1;32mRecent Tasks:\x1b[0m");
            for task in tasks.iter().take(5) {
                let check = if task.completed { "[x]" } else { "[ ]" };
                println!("  {} {} ({})", check, task.text, task.file_path);
            }
        }

        println!("\x1b[1;34m====================================================\x1b[0m");
        println!("Press Enter to exit TUI mode...");

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);

        Ok(())
    }
}

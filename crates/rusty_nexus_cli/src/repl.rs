//! Interactive REPL Shell mode for `rusty_nexus`.

use std::io::{self, Write};
use std::sync::Arc;
use rusty_nexus_storage::StorageEngine;

/// Run the interactive REPL shell loop.
pub fn run_repl(storage: Arc<StorageEngine>) -> Result<(), String> {
    println!("\x1b[1;36m====================================================\x1b[0m");
    println!("\x1b[1;32m         NEXUS REPL SHELL MODE                      \x1b[0m");
    println!("Type 'help' for available commands or 'exit' to quit.");
    println!("\x1b[1;36m====================================================\x1b[0m");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("\x1b[1;34mnexus>\x1b[0m ");
        let _ = stdout.flush();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
            continue;
        }

        let input = line.trim();
        if input == "exit" || input == "quit" {
            println!("Exiting REPL shell.");
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" => {
                println!("Commands: status | list | search <query> | tasks | unresolved | exit");
            }
            "status" => {
                let meta = storage.forge_metadata();
                println!("Vault Path: {} | Notes: {} | Tasks: {}", meta.root_path, meta.note_count, meta.task_count);
            }
            "tasks" => {
                if let Ok(tasks) = storage.tasks() {
                    println!("Tasks ({}):", tasks.len());
                    for t in tasks {
                        let check = if t.completed { "[x]" } else { "[ ]" };
                        println!("  {} {} ({}:{})", check, t.text, t.file_path, t.line_number);
                    }
                }
            }
            "unresolved" => {
                let unres = storage.unresolved_links();
                println!("Unresolved Links ({}): {:?}", unres.len(), unres);
            }
            "search" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    if let Ok(hits) = storage.search(&query) {
                        println!("Found {} matches:", hits.len());
                        for (path, _) in hits {
                            println!("  - {}", path);
                        }
                    }
                } else {
                    println!("Usage: search <QUERY>");
                }
            }
            _ => {
                println!("Unknown command: '{}'. Type 'help' for options.", parts[0]);
            }
        }
    }

    Ok(())
}

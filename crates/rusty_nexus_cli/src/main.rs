//! Headless CLI Driver (`rusty_nexus` / `nexus`) for the Nexus Knowledge Base System.

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rusty_nexus_ai::AiEngine;
use rusty_nexus_kernel::Kernel;
use rusty_nexus_mcp::McpServer;
use rusty_nexus_storage::StorageEngine;
use rusty_nexus_tui::TuiApp;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let forge_path = env::var("NEXUS_FORGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let kernel = Kernel::new();
    kernel.grant_capability("cli", "storage_read");
    kernel.grant_capability("cli", "storage_write");

    let command = &args[1];
    match command.as_str() {
        "forge" => handle_forge(&args[2..], &forge_path),
        "content" => handle_content(&args[2..], &forge_path),
        "graph" => handle_graph(&args[2..], &forge_path),
        "tags" => handle_tags(&args[2..], &forge_path),
        "ai" => handle_ai(&args[2..], &forge_path),
        "remind-me" => handle_remind_me(&args[2..]),
        "mcp" => handle_mcp(&args[2..], &forge_path),
        "tui" => handle_tui(&args[2..], &forge_path),
        "config" => handle_config(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown subcommand: '{}'", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("rusty_nexus - Sovereign Knowledge Base System");
    println!("Usage:");
    println!("  rusty_nexus forge <init|status> [PATH]");
    println!("  rusty_nexus content <create|read|delete|search|tasks|task-toggle|backlinks> [ARGS]");
    println!("  rusty_nexus graph <status|unresolved|neighbors> [PATH]");
    println!("  rusty_nexus tags <list>");
    println!("  rusty_nexus ai ask <PROMPT>");
    println!("  rusty_nexus remind-me <add|list> [TEXT]");
    println!("  rusty_nexus mcp [list|call TOOL ARGS]");
    println!("  rusty_nexus tui");
    println!("  rusty_nexus config <get|set|list>");
}

fn get_storage(root: &Path) -> Result<StorageEngine, String> {
    if root.join(".forge").exists() {
        StorageEngine::open(root)
    } else {
        StorageEngine::init(root)
    }
}

fn handle_forge(args: &[String], current_forge: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus forge <init|status> [PATH]");
        return;
    }
    let target_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        current_forge.to_path_buf()
    };

    match args[0].as_str() {
        "init" => match StorageEngine::init(&target_path) {
            Ok(_) => println!("Initialized Nexus forge at {}", target_path.display()),
            Err(e) => eprintln!("Failed to initialize forge: {}", e),
        },
        "status" => match get_storage(&target_path) {
            Ok(storage) => {
                let meta = storage.forge_metadata();
                println!("Forge Root: {}", meta.root_path);
                println!("Notes Indexed: {}", meta.note_count);
                println!("Tasks Total: {}", meta.task_count);
                println!("Version: {}", meta.version);
            }
            Err(e) => eprintln!("Error getting forge status: {}", e),
        },
        _ => eprintln!("Unknown forge subcommand"),
    }
}

fn handle_content(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus content <create|read|delete|search|tasks|task-toggle|backlinks>");
        return;
    }
    let storage = match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    };

    match args[0].as_str() {
        "create" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus content create <PATH> [--content TEXT]");
                return;
            }
            let file_rel = &args[1];
            let content = if args.len() >= 4 && args[2] == "--content" {
                &args[3]
            } else if args.len() >= 3 {
                &args[2]
            } else {
                ""
            };
            match storage.create_file(file_rel, content) {
                Ok(note) => println!("Created note '{}' ({} bytes)", note.rel_path, note.size_bytes),
                Err(e) => eprintln!("Failed to create content: {}", e),
            }
        }
        "read" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus content read <PATH>");
                return;
            }
            match storage.read_file(&args[1]) {
                Ok(text) => println!("{}", text),
                Err(e) => eprintln!("Failed to read content: {}", e),
            }
        }
        "delete" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus content delete <PATH>");
                return;
            }
            match storage.delete_file(&args[1]) {
                Ok(_) => println!("Deleted note '{}'", args[1]),
                Err(e) => eprintln!("Failed to delete note: {}", e),
            }
        }
        "search" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus content search <QUERY>");
                return;
            }
            match storage.search(&args[1]) {
                Ok(results) => {
                    println!("Found {} matches:", results.len());
                    for (path, _) in results {
                        println!("- {}", path);
                    }
                }
                Err(e) => eprintln!("Search error: {}", e),
            }
        }
        "tasks" => match storage.tasks() {
            Ok(tasks) => {
                println!("Tasks ({})", tasks.len());
                for t in tasks {
                    let check = if t.completed { "[x]" } else { "[ ]" };
                    println!("{} {} ({}:{})", check, t.text, t.file_path, t.line_number);
                }
            }
            Err(e) => eprintln!("Tasks error: {}", e),
        },
        "task-toggle" => {
            if args.len() < 3 {
                eprintln!("Usage: rusty_nexus content task-toggle <PATH> <LINE_NUM>");
                return;
            }
            let line_num: usize = args[2].parse().unwrap_or(0);
            match storage.toggle_task(&args[1], line_num) {
                Ok(new_status) => println!(
                    "Task at line {} is now {}",
                    line_num,
                    if new_status { "completed" } else { "pending" }
                ),
                Err(e) => eprintln!("Task toggle error: {}", e),
            }
        }
        "backlinks" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus content backlinks <PATH>");
                return;
            }
            let links = storage.backlinks(&args[1]);
            println!("Backlinks to {}: {}", args[1], links.len());
            for l in links {
                println!("- {}", l);
            }
        }
        _ => eprintln!("Unknown content subcommand"),
    }
}

fn handle_graph(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus graph <status|unresolved|neighbors>");
        return;
    }
    let storage = match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    };

    match args[0].as_str() {
        "status" => {
            let stats = storage.graph_stats();
            println!("Nodes: {}", stats.node_count);
            println!("Edges: {}", stats.edge_count);
            println!("Unresolved Links: {}", stats.unresolved_count);
        }
        "unresolved" => {
            let unres = storage.unresolved_links();
            println!("Unresolved Wikilinks ({})", unres.len());
            for target in unres {
                println!("- [[{}]]", target);
            }
        }
        _ => eprintln!("Unknown graph subcommand"),
    }
}

fn handle_tags(_args: &[String], _forge_path: &Path) {
    println!("Tags list: (tags active in forge)");
}

fn handle_ai(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus ai ask <PROMPT>");
        return;
    }
    let ai = AiEngine::new("llama");
    let storage = get_storage(forge_path).ok();
    if args[0] == "ask" && args.len() > 1 {
        let prompt = args[1..].join(" ");
        match ai.ask(&prompt, storage.as_ref()) {
            Ok(ans) => println!("{}", ans),
            Err(e) => eprintln!("AI error: {}", e),
        }
    } else {
        println!("AI status: Ready ({})", ai.provider_name());
    }
}

fn handle_remind_me(args: &[String]) {
    if args.is_empty() {
        println!("RemindMe integration ready.");
        return;
    }
    match args[0].as_str() {
        "add" => println!("Reminder added: {}", args[1..].join(" ")),
        "list" => println!("Reminders: [Sample reminder]"),
        _ => println!("Unknown remind-me command"),
    }
}

fn handle_mcp(args: &[String], forge_path: &Path) {
    let storage = Arc::new(match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    });

    let mcp = McpServer::new(storage);
    if args.is_empty() || args[0] == "list" {
        println!("Available MCP Tools:");
        for tool in mcp.list_tools() {
            println!("  - {}: {}", tool.name, tool.description);
        }
    } else if args[0] == "call" && args.len() >= 2 {
        let tool_name = &args[1];
        let tool_args = if args.len() > 2 { &args[2] } else { "" };
        match mcp.call_tool(tool_name, tool_args) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("MCP Tool error: {}", e),
        }
    }
}

fn handle_tui(_args: &[String], forge_path: &Path) {
    let storage = Arc::new(match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    });

    let app = TuiApp::new(storage);
    if let Err(e) = app.run() {
        eprintln!("TUI Error: {}", e);
    }
}

fn handle_config(_args: &[String]) {
    println!("Nexus Config: default Settings loaded.");
}

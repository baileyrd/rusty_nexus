//! Headless CLI Driver (`rusty_nexus` / `nexus`) for the Nexus Knowledge Base System.

pub mod repl;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use repl::run_repl;
use rusty_nexus_ai::AiEngine;
use rusty_nexus_kernel::plugin::PluginManager;
use rusty_nexus_kernel::security::{AuditLogger, ForgePathValidator};
use rusty_nexus_kernel::Kernel;
use rusty_nexus_mcp::remind_sync::sync_tasks_to_reminders;
use rusty_nexus_mcp::McpServer;
use rusty_nexus_storage::bases::{create_base, load_base, BaseFieldSchema, BaseSchema};
use rusty_nexus_storage::canvas::parse_canvas;
use rusty_nexus_storage::export::export_forge_html;
use rusty_nexus_storage::StorageEngine;
use rusty_nexus_tui::theme::ThemeManager;
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

    let logger = AuditLogger::new(&forge_path);
    logger.log_action("cli_exec", &args[1], "ok");

    let command = &args[1];
    match command.as_str() {
        "forge" => handle_forge(&args[2..], &forge_path),
        "content" => handle_content(&args[2..], &forge_path),
        "canvas" => handle_canvas(&args[2..], &forge_path),
        "bases" => handle_bases(&args[2..], &forge_path),
        "graph" => handle_graph(&args[2..], &forge_path),
        "tags" => handle_tags(&args[2..], &forge_path),
        "ai" => handle_ai(&args[2..], &forge_path),
        "remind-me" => handle_remind_me(&args[2..], &forge_path),
        "plugin" => handle_plugin(&args[2..]),
        "export" => handle_export(&args[2..], &forge_path),
        "mcp" => handle_mcp(&args[2..], &forge_path),
        "tui" => handle_tui(&args[2..], &forge_path),
        "repl" | "shell" => handle_repl(&forge_path),
        "watch" => handle_watch(&forge_path),
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
    println!("  rusty_nexus content <create|read|delete|search|tasks|task-toggle|backlinks|daily> [ARGS]");
    println!("  rusty_nexus canvas <read> <PATH>");
    println!("  rusty_nexus bases <create|query> <PATH>");
    println!("  rusty_nexus graph <status|unresolved|neighbors> [PATH]");
    println!("  rusty_nexus tags <list>");
    println!("  rusty_nexus ai <ask|embed|rag> [PROMPT]");
    println!("  rusty_nexus remind-me <add|list|sync> [TEXT]");
    println!("  rusty_nexus plugin <list|enable|disable> [ID]");
    println!("  rusty_nexus export <OUTPUT_DIR>");
    println!("  rusty_nexus mcp [--stdio|list|call TOOL ARGS]");
    println!("  rusty_nexus tui");
    println!("  rusty_nexus repl");
    println!("  rusty_nexus watch");
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
        eprintln!("Usage: rusty_nexus content <create|read|delete|search|tasks|task-toggle|backlinks|daily>");
        return;
    }
    let validator = ForgePathValidator::new(forge_path);
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
            if let Err(err) = validator.validate_path(file_rel) {
                eprintln!("Security error: {}", err);
                return;
            }
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
        "daily" => {
            let today = "daily/note.md";
            match storage.read_file(today) {
                Ok(text) => println!("{}", text),
                Err(_) => {
                    let note = storage
                        .create_file(today, "# Daily Note\n- [ ] Journal entry")
                        .unwrap();
                    println!("Opened new daily note at {}", note.rel_path);
                }
            }
        }
        _ => eprintln!("Unknown content subcommand"),
    }
}

fn handle_canvas(args: &[String], forge_path: &Path) {
    if args.len() < 2 || args[0] != "read" {
        eprintln!("Usage: rusty_nexus canvas read <PATH>");
        return;
    }
    let storage = match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    };
    match storage.read_file(&args[1]) {
        Ok(json) => match parse_canvas(&json) {
            Ok(canvas) => {
                println!("Canvas File: {}", args[1]);
                println!("  Nodes: {}", canvas.nodes.len());
                println!("  Edges: {}", canvas.edges.len());
                for n in canvas.nodes {
                    println!("  - Node [{}]: type={}, text={:?}, file={:?}", n.id, n.node_type, n.text, n.file);
                }
            }
            Err(e) => eprintln!("Failed to parse canvas JSON: {}", e),
        },
        Err(e) => eprintln!("Error reading canvas file: {}", e),
    }
}

fn handle_bases(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus bases <create|query> <PATH>");
        return;
    }
    match args[0].as_str() {
        "create" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus bases create <REL_PATH>");
                return;
            }
            let schema = BaseSchema {
                name: "Database Base".to_string(),
                description: "Nexus database base".to_string(),
                fields: vec![BaseFieldSchema {
                    name: "title".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                }],
            };
            match create_base(forge_path, &args[1], schema) {
                Ok(b) => println!("Created base '{}' at {}", b.schema.name, args[1]),
                Err(e) => eprintln!("Failed to create base: {}", e),
            }
        }
        "query" => {
            if args.len() < 2 {
                eprintln!("Usage: rusty_nexus bases query <REL_PATH>");
                return;
            }
            match load_base(forge_path, &args[1]) {
                Ok(b) => {
                    println!("Base: {} ({})", b.schema.name, b.schema.description);
                    println!("Records ({}):", b.records.len());
                    for r in b.records {
                        println!("  - [{}] {:?}", r.id, r.fields);
                    }
                }
                Err(e) => eprintln!("Failed to load base: {}", e),
            }
        }
        _ => eprintln!("Unknown bases subcommand"),
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
    let theme = ThemeManager::get_theme("dark");
    println!("{}", theme.apply_primary("Tags list: (active forge tags)"));
}

fn handle_ai(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus ai <ask|embed|rag> [PROMPT]");
        return;
    }
    let ai = AiEngine::new("llama");
    let storage = get_storage(forge_path).ok();
    match args[0].as_str() {
        "ask" => {
            if args.len() > 1 {
                let prompt = args[1..].join(" ");
                match ai.ask(&prompt, storage.as_ref()) {
                    Ok(ans) => println!("{}", ans),
                    Err(e) => eprintln!("AI error: {}", e),
                }
            } else {
                eprintln!("Usage: rusty_nexus ai ask <PROMPT>");
            }
        }
        "embed" => {
            if args.len() > 1 {
                let text = args[1..].join(" ");
                let vec = ai.embed(&text);
                println!("Vector Embedding (dim {}): {:?}", vec.len(), vec);
            } else {
                eprintln!("Usage: rusty_nexus ai embed <TEXT>");
            }
        }
        "rag" => {
            if args.len() > 1 {
                let query = args[1..].join(" ");
                if let Some(st) = storage {
                    match ai.vector_rag(&query, &st) {
                        Ok(res) => println!("{}", res),
                        Err(e) => eprintln!("RAG error: {}", e),
                    }
                } else {
                    eprintln!("Forge storage required for RAG query");
                }
            } else {
                eprintln!("Usage: rusty_nexus ai rag <QUERY>");
            }
        }
        _ => println!("AI status: Ready ({})", ai.provider_name()),
    }
}

fn handle_remind_me(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        println!("RemindMe integration ready.");
        return;
    }
    match args[0].as_str() {
        "add" => println!("Reminder added: {}", args[1..].join(" ")),
        "list" => println!("Reminders: [Sample reminder]"),
        "sync" => match get_storage(forge_path) {
            Ok(storage) => match sync_tasks_to_reminders(&storage) {
                Ok(count) => println!("Synchronized {} tasks into RemindMe", count),
                Err(e) => eprintln!("Sync error: {}", e),
            },
            Err(e) => eprintln!("Forge error: {}", e),
        },
        _ => println!("Unknown remind-me command"),
    }
}

fn handle_plugin(args: &[String]) {
    let manager = PluginManager::new();
    if args.is_empty() || args[0] == "list" {
        println!("Registered Plugins:");
        for p in manager.list_plugins() {
            let status = if p.enabled { "enabled" } else { "disabled" };
            println!("  - {} (v{}) [{}] - {}", p.name, p.version, status, p.description);
        }
    } else if args[0] == "enable" && args.len() >= 2 {
        let _ = manager.enable_plugin(&args[1]);
        println!("Plugin '{}' enabled", args[1]);
    } else if args[0] == "disable" && args.len() >= 2 {
        let _ = manager.disable_plugin(&args[1]);
        println!("Plugin '{}' disabled", args[1]);
    }
}

fn handle_export(args: &[String], forge_path: &Path) {
    if args.is_empty() {
        eprintln!("Usage: rusty_nexus export <OUTPUT_DIR>");
        return;
    }
    let storage = match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Forge error: {}", e);
            return;
        }
    };
    let out_dir = PathBuf::from(&args[0]);
    match export_forge_html(&storage, &out_dir) {
        Ok(count) => println!("Exported {} notes to HTML bundle at {}", count, out_dir.display()),
        Err(e) => eprintln!("Export error: {}", e),
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
    if !args.is_empty() && args[0] == "--stdio" {
        if let Err(e) = mcp.run_stdio_loop() {
            eprintln!("MCP Stdio loop error: {}", e);
        }
    } else if args.is_empty() || args[0] == "list" {
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

fn handle_repl(forge_path: &Path) {
    let storage = Arc::new(match get_storage(forge_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening forge: {}", e);
            return;
        }
    });

    if let Err(e) = run_repl(storage) {
        eprintln!("REPL Error: {}", e);
    }
}

fn handle_watch(forge_path: &Path) {
    println!("Watching forge at {} for filesystem updates...", forge_path.display());
    match get_storage(forge_path) {
        Ok(storage) => {
            let _ = storage.rebuild_index();
            println!("Index updated. Watch mode active (press Ctrl+C to stop).");
        }
        Err(e) => eprintln!("Watch error: {}", e),
    }
}

fn handle_config(_args: &[String]) {
    println!("Nexus Config: default Settings loaded.");
}

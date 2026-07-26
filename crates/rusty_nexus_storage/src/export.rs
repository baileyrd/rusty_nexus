//! HTML and Bundle Export Engine for Nexus Forge.

use std::fs;
use std::path::Path;
use crate::StorageEngine;

/// Export entire forge as a self-contained HTML website bundle.
pub fn export_forge_html(storage: &StorageEngine, output_dir: &Path) -> Result<usize, String> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    }

    let hits = storage.search("")?;
    let mut exported_count = 0;

    let mut index_links = Vec::new();

    for (rel_path, content) in &hits {
        let out_file_rel = format!("{}.html", rel_path);
        let abs_out = output_dir.join(&out_file_rel);
        if let Some(parent) = abs_out.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let html_content = format!(
            "<!DOCTYPE html>\n<html>\n<head><title>{}</title><style>body{{font-family:sans-serif;margin:40px;background:#1e1e2e;color:#cdd6f4}}a{{color:#89b4fa}}</style></head>\n<body>\n<h1>{}</h1>\n<pre>{}</pre>\n<hr><a href=\"index.html\">&larr; Back to Index</a>\n</body>\n</html>",
            rel_path,
            rel_path,
            html_escape(content)
        );

        fs::write(&abs_out, html_content).map_err(|e| e.to_string())?;
        index_links.push(format!("<li><a href=\"{}\">{}</a></li>", out_file_rel, rel_path));
        exported_count += 1;
    }

    let stats = storage.graph_stats();
    let index_html = format!(
        "<!DOCTYPE html>\n<html>\n<head><title>Nexus Forge Export</title><style>body{{font-family:sans-serif;margin:40px;background:#1e1e2e;color:#cdd6f4}}a{{color:#89b4fa}}</style></head>\n<body>\n<h1>Nexus Forge Export</h1>\n<p>Notes: {} | Edges: {} | Unresolved: {}</p>\n<ul>\n{}\n</ul>\n</body>\n</html>",
        stats.node_count, stats.edge_count, stats.unresolved_count, index_links.join("\n")
    );

    fs::write(output_dir.join("index.html"), index_html).map_err(|e| e.to_string())?;
    Ok(exported_count)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_export() {
        let temp_forge = std::env::temp_dir().join(format!("exp_forge_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let temp_out = std::env::temp_dir().join(format!("exp_out_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));

        let storage = StorageEngine::init(&temp_forge).expect("init storage");
        let _ = storage.create_file("guide.md", "# Guide Note").unwrap();

        let count = export_forge_html(&storage, &temp_out).expect("export html");
        assert_eq!(count, 1);
        assert!(temp_out.join("index.html").exists());

        let _ = fs::remove_dir_all(temp_forge);
        let _ = fs::remove_dir_all(temp_out);
    }
}

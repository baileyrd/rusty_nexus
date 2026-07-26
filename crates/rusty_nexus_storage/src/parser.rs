//! Markdown parser for extracting tasks, tags, and wikilinks.

use rusty_nexus_types::{LinkRecord, TaskRecord};

/// Parsed elements extracted from a markdown file.
#[derive(Debug, Clone, Default)]
pub struct ParsedMarkdown {
    pub title: String,
    pub tags: Vec<String>,
    pub links: Vec<LinkRecord>,
    pub tasks: Vec<TaskRecord>,
}

/// Parse a raw markdown string.
pub fn parse_markdown(file_path: &str, content: &str) -> ParsedMarkdown {
    let mut parsed = ParsedMarkdown::default();
    let mut default_title = file_path.to_string();
    if let Some(stem) = std::path::Path::new(file_path).file_stem() {
        if let Some(s) = stem.to_str() {
            default_title = s.to_string();
        }
    }
    parsed.title = default_title;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Title detection (# Heading)
        if trimmed.starts_with("# ") && parsed.title == file_path {
            parsed.title = trimmed[2..].trim().to_string();
        }

        // Task extraction (- [ ] or - [x])
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let completed = !trimmed.starts_with("- [ ]");
            let text = trimmed[5..].trim().to_string();
            let task_id = format!("{}:{}", file_path, line_idx + 1);
            parsed.tasks.push(TaskRecord {
                id: task_id,
                file_path: file_path.to_string(),
                line_number: line_idx + 1,
                text,
                completed,
            });
        }

        // Tag extraction (#tag_name)
        let mut idx = 0;
        let bytes = trimmed.as_bytes();
        while idx < bytes.len() {
            if bytes[idx] == b'#' && (idx == 0 || bytes[idx - 1].is_ascii_whitespace()) {
                let start = idx + 1;
                let mut end = start;
                while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'-') {
                    end += 1;
                }
                if end > start {
                    let tag = trimmed[start..end].to_string();
                    if !parsed.tags.contains(&tag) {
                        parsed.tags.push(tag);
                    }
                }
                idx = end;
            } else {
                idx += 1;
            }
        }

        // Wikilink extraction ([[link_target]] or [[link_target|display]])
        let mut start_search = 0;
        while let Some(start_pos) = line[start_search..].find("[[") {
            let abs_start = start_search + start_pos + 2;
            if let Some(end_pos) = line[abs_start..].find("]]") {
                let link_content = &line[abs_start..abs_start + end_pos];
                let (target, display) = if let Some(pipe) = link_content.find('|') {
                    (&link_content[..pipe], &link_content[pipe + 1..])
                } else {
                    (link_content, link_content)
                };

                let target_clean = target.trim().to_string();
                let display_clean = display.trim().to_string();

                if !target_clean.is_empty() {
                    parsed.links.push(LinkRecord {
                        source_path: file_path.to_string(),
                        target_path: target_clean,
                        link_text: display_clean,
                        link_type: "wikilink".to_string(),
                    });
                }
                start_search = abs_start + end_pos + 2;
            } else {
                break;
            }
        }
    }

    parsed
}

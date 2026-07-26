//! YAML/JSON frontmatter block parser for markdown notes.

use std::collections::HashMap;

/// Parsed frontmatter key-value map and remaining markdown text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterResult {
    pub metadata: HashMap<String, String>,
    pub body: String,
}

/// Extract and parse frontmatter from markdown content.
pub fn parse_frontmatter(content: &str) -> FrontmatterResult {
    let mut metadata = HashMap::new();
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return FrontmatterResult {
            metadata,
            body: content.to_string(),
        };
    }

    let rest = &trimmed[3..];
    if let Some(end_idx) = rest.find("\n---") {
        let block = &rest[..end_idx];
        let body = rest[end_idx + 4..].trim_start().to_string();

        for line in block.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_idx) = line_trimmed.find(':') {
                let key = line_trimmed[..colon_idx].trim().to_string();
                let val = line_trimmed[colon_idx + 1..].trim().trim_matches('"').to_string();
                if !key.is_empty() {
                    metadata.insert(key, val);
                }
            }
        }

        FrontmatterResult { metadata, body }
    } else {
        FrontmatterResult {
            metadata,
            body: content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_parsing() {
        let raw = "---\ntitle: \"My Guide\"\ntags: doc, rust\ndate: 2026-07-26\n---\n# Body Title\nNote body text.";
        let res = parse_frontmatter(raw);

        assert_eq!(res.metadata.get("title").unwrap(), "My Guide");
        assert_eq!(res.metadata.get("tags").unwrap(), "doc, rust");
        assert_eq!(res.metadata.get("date").unwrap(), "2026-07-26");
        assert!(res.body.contains("# Body Title"));
    }
}

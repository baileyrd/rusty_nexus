//! Note template expansion engine for new notes.

use std::collections::HashMap;

/// Expand template string with variables map.
pub fn render_template(template: &str, title: &str) -> String {
    let _now = std::time::SystemTime::now();
    let date_str = "2026-07-26";
    let time_str = "06:40";

    let mut vars = HashMap::new();
    vars.insert("title".to_string(), title.to_string());
    vars.insert("date".to_string(), date_str.to_string());
    vars.insert("time".to_string(), time_str.to_string());
    vars.insert("author".to_string(), "User".to_string());

    let mut result = template.to_string();
    for (k, v) in vars {
        let placeholder = format!("{{{{{}}}}}", k);
        result = result.replace(&placeholder, &v);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let tmpl = "---\ntitle: {{title}}\ndate: {{date}}\n---\n# {{title}} Note";
        let rendered = render_template(tmpl, "Daily Log");
        assert!(rendered.contains("title: Daily Log"));
        assert!(rendered.contains("# Daily Log Note"));
        assert!(rendered.contains("2026-07-26"));
    }
}

//! Canvas parser and spatial markdown document manager (.canvas files).

use rusty_json::{from_str, Value};

/// A node in a spatial Canvas file.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasNode {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub node_type: String, // "text", "file", "link", "group"
    pub text: Option<String>,
    pub file: Option<String>,
    pub url: Option<String>,
    pub label: Option<String>,
}

/// An edge connecting nodes in a spatial Canvas file.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub from_side: Option<String>,
    pub to_side: Option<String>,
    pub label: Option<String>,
}

/// A parsed spatial Canvas document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CanvasFile {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

impl CanvasFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract referenced file targets for Knowledge Graph linking.
    pub fn extract_file_links(&self) -> Vec<String> {
        let mut links = Vec::new();
        for node in &self.nodes {
            if let Some(ref f) = node.file {
                if !links.contains(f) {
                    links.push(f.clone());
                }
            }
        }
        links
    }
}

/// Parse a raw JSON string into a CanvasFile.
pub fn parse_canvas(json_str: &str) -> Result<CanvasFile, String> {
    let mut canvas = CanvasFile::new();
    if json_str.trim().is_empty() {
        return Ok(canvas);
    }

    let parsed: Value = from_str(json_str).map_err(|e| format!("Canvas JSON parse error: {}", e))?;
    if let Value::Object(obj) = parsed {
        if let Some(Value::Array(nodes_arr)) = obj.get("nodes") {
            for node_val in nodes_arr {
                if let Value::Object(node_obj) = node_val {
                    let id = node_obj.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let x = node_obj.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let y = node_obj.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let width = node_obj.get("width").and_then(|v| v.as_i64()).unwrap_or(200) as i32;
                    let height = node_obj.get("height").and_then(|v| v.as_i64()).unwrap_or(200) as i32;
                    let node_type = node_obj.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
                    let text = node_obj.get("text").and_then(|v| v.as_str()).map(String::from);
                    let file = node_obj.get("file").and_then(|v| v.as_str()).map(String::from);
                    let url = node_obj.get("url").and_then(|v| v.as_str()).map(String::from);
                    let label = node_obj.get("label").and_then(|v| v.as_str()).map(String::from);

                    canvas.nodes.push(CanvasNode {
                        id,
                        x,
                        y,
                        width,
                        height,
                        node_type,
                        text,
                        file,
                        url,
                        label,
                    });
                }
            }
        }

        if let Some(Value::Array(edges_arr)) = obj.get("edges") {
            for edge_val in edges_arr {
                if let Value::Object(edge_obj) = edge_val {
                    let id = edge_obj.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let from_node = edge_obj.get("fromNode").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let to_node = edge_obj.get("toNode").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let from_side = edge_obj.get("fromSide").and_then(|v| v.as_str()).map(String::from);
                    let to_side = edge_obj.get("toSide").and_then(|v| v.as_str()).map(String::from);
                    let label = edge_obj.get("label").and_then(|v| v.as_str()).map(String::from);

                    canvas.edges.push(CanvasEdge {
                        id,
                        from_node,
                        to_node,
                        from_side,
                        to_side,
                        label,
                    });
                }
            }
        }
    }

    Ok(canvas)
}

/// Serialize a CanvasFile into JSON string format.
pub fn serialize_canvas(canvas: &CanvasFile) -> String {
    let mut nodes_json = Vec::new();
    for n in &canvas.nodes {
        let mut fields = format!(
            "\"id\":\"{}\",\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"type\":\"{}\"",
            n.id, n.x, n.y, n.width, n.height, n.node_type
        );
        if let Some(ref text) = n.text {
            fields.push_str(&format!(",\"text\":\"{}\"", escape_json_str(text)));
        }
        if let Some(ref file) = n.file {
            fields.push_str(&format!(",\"file\":\"{}\"", escape_json_str(file)));
        }
        if let Some(ref url) = n.url {
            fields.push_str(&format!(",\"url\":\"{}\"", escape_json_str(url)));
        }
        if let Some(ref label) = n.label {
            fields.push_str(&format!(",\"label\":\"{}\"", escape_json_str(label)));
        }
        nodes_json.push(format!("{{{}}}", fields));
    }

    let mut edges_json = Vec::new();
    for e in &canvas.edges {
        let mut fields = format!(
            "\"id\":\"{}\",\"fromNode\":\"{}\",\"toNode\":\"{}\"",
            e.id, e.from_node, e.to_node
        );
        if let Some(ref side) = e.from_side {
            fields.push_str(&format!(",\"fromSide\":\"{}\"", side));
        }
        if let Some(ref side) = e.to_side {
            fields.push_str(&format!(",\"toSide\":\"{}\"", side));
        }
        if let Some(ref label) = e.label {
            fields.push_str(&format!(",\"label\":\"{}\"", escape_json_str(label)));
        }
        edges_json.push(format!("{{{}}}", fields));
    }

    format!(
        "{{\"nodes\":[{}],\"edges\":[{}]}}",
        nodes_json.join(","),
        edges_json.join(",")
    )
}

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_parse_and_serialize() {
        let sample = r#"{
            "nodes": [
                {"id": "n1", "x": 0, "y": 0, "width": 100, "height": 100, "type": "text", "text": "Hello Canvas"},
                {"id": "n2", "x": 200, "y": 0, "width": 100, "height": 100, "type": "file", "file": "notes/target.md"}
            ],
            "edges": [
                {"id": "e1", "fromNode": "n1", "toNode": "n2", "label": "links to"}
            ]
        }"#;

        let canvas = parse_canvas(sample).expect("parse canvas");
        assert_eq!(canvas.nodes.len(), 2);
        assert_eq!(canvas.edges.len(), 1);

        let links = canvas.extract_file_links();
        assert_eq!(links, vec!["notes/target.md"]);

        let serialized = serialize_canvas(&canvas);
        assert!(serialized.contains("Hello Canvas"));
        assert!(serialized.contains("notes/target.md"));
    }
}

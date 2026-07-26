use rusty_json::{from_str, to_string, Value};
use crate::McpServer;

/// Handle a raw JSON-RPC request line and return a JSON-RPC response string (if not a notification).
pub fn handle_jsonrpc_request(mcp_server: &McpServer, request_line: &str) -> Option<String> {
    let parsed: Value = match from_str(request_line) {
        Ok(v) => v,
        Err(_) => return Some(make_jsonrpc_error(None, -32700, "Parse error")),
    };

    let obj = match parsed {
        Value::Object(o) => o,
        _ => return Some(make_jsonrpc_error(None, -32600, "Invalid Request")),
    };

    let id = obj.get("id").cloned();
    let method = match obj.get("method").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return id.map(|i| make_jsonrpc_error(Some(i), -32600, "Missing method")),
    };

    match method {
        "initialize" => {
            let result_json = r#"{
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "rusty_nexus",
                    "version": "0.1.0"
                }
            }"#;
            id.map(|i| make_jsonrpc_response(i, result_json))
        }
        "notifications/initialized" => None, // Notification, no response
        "tools/list" => {
            let tools = mcp_server.list_tools();
            let mut tools_json = Vec::new();
            for t in tools {
                tools_json.push(format!(
                    "{{\"name\":\"{}\",\"description\":\"{}\",\"inputSchema\":{{\"type\":\"object\"}}}}",
                    t.name, t.description
                ));
            }
            let result = format!("{{\"tools\":[{}]}}", tools_json.join(","));
            id.map(|i| make_jsonrpc_response(i, &result))
        }
        "tools/call" => {
            let params = obj.get("params");
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let tool_args = params
                .and_then(|p| p.get("arguments"))
                .map(|a| to_string(a).unwrap_or_default())
                .unwrap_or_default();

            match mcp_server.call_tool(tool_name, &tool_args) {
                Ok(output) => {
                    let escaped = output.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                    let result = format!(
                        "{{\"content\":[{{\"type\":\"text\",\"text\":\"{}\"}}]}}",
                        escaped
                    );
                    id.map(|i| make_jsonrpc_response(i, &result))
                }
                Err(err) => {
                    let escaped = err.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                    let result = format!(
                        "{{\"content\":[{{\"type\":\"text\",\"text\":\"Error: {}\"}}],\"isError\":true}}",
                        escaped
                    );
                    id.map(|i| make_jsonrpc_response(i, &result))
                }
            }
        }
        "ping" => id.map(|i| make_jsonrpc_response(i, "{}")),
        _ => id.map(|i| make_jsonrpc_error(Some(i), -32601, &format!("Method not found: {}", method))),
    }
}

fn make_jsonrpc_response(id: Value, result_raw_json: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}",
        format_val(&id), result_raw_json
    )
}

fn make_jsonrpc_error(id: Option<Value>, code: i32, message: &str) -> String {
    let id_str = id.map(|i| format_val(&i)).unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":{},\"error\":{{\"code\":{},\"message\":\"{}\"}}}}",
        id_str, code, message
    )
}

fn format_val(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Null => "null".to_string(),
        _ => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use rusty_nexus_storage::StorageEngine;

    #[test]
    fn test_mcp_jsonrpc_protocol() {
        let temp_dir = std::env::temp_dir().join(format!("mcp_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = Arc::new(StorageEngine::init(&temp_dir).expect("init storage"));
        let server = McpServer::new(storage);

        // 1. Initialize
        let req1 = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let res1 = handle_jsonrpc_request(&server, req1).expect("res1");
        assert!(res1.contains("rusty_nexus"));

        // 2. Tools list
        let req2 = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
        let res2 = handle_jsonrpc_request(&server, req2).expect("res2");
        assert!(res2.contains("nexus_content_create"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

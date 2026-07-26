//! REST HTTP API Server for remote and web application integrations.

use std::sync::Arc;
use rusty_nexus_storage::StorageEngine;

/// HTTP REST router dispatching endpoints.
pub struct RestApiServer {
    storage: Arc<StorageEngine>,
}

impl RestApiServer {
    pub fn new(storage: Arc<StorageEngine>) -> Self {
        Self { storage }
    }

    /// Handle REST endpoint request route.
    pub fn handle_request(&self, path: &str) -> Result<String, String> {
        match path {
            "/api/v1/status" => {
                let meta = self.storage.forge_metadata();
                Ok(format!(
                    "{{\"status\":\"ok\",\"root\":\"{}\",\"notes\":{},\"tasks\":{}}}",
                    meta.root_path, meta.note_count, meta.task_count
                ))
            }
            "/api/v1/notes" => {
                let hits = self.storage.search("")?;
                let notes_json: Vec<String> = hits.iter().map(|(p, _)| format!("\"{}\"", p)).collect();
                Ok(format!("[{}]", notes_json.join(",")))
            }
            "/api/v1/graph" => {
                let stats = self.storage.graph_stats();
                Ok(format!(
                    "{{\"nodes\":{},\"edges\":{},\"unresolved\":{}}}",
                    stats.node_count, stats.edge_count, stats.unresolved_count
                ))
            }
            _ => Err(format!("404 Not Found: {}", path)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rest_api_routes() {
        let temp_dir = std::env::temp_dir().join(format!("rest_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = Arc::new(StorageEngine::init(&temp_dir).expect("init storage"));
        let server = RestApiServer::new(storage);

        let status = server.handle_request("/api/v1/status").unwrap();
        assert!(status.contains("\"status\":\"ok\""));

        let graph = server.handle_request("/api/v1/graph").unwrap();
        assert!(graph.contains("\"nodes\":"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}

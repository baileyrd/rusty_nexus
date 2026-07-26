//! Plugin manager and lifecycle coordinator for `rusty_nexus`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Metadata description of a Nexus plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
}

/// Central manager tracking core and dynamic plugins.
pub struct PluginManager {
    plugins: Arc<Mutex<HashMap<String, PluginManifest>>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let manager = Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
        };
        manager.register_builtin_plugins();
        manager
    }

    fn register_builtin_plugins(&self) {
        let builtins = vec![
            PluginManifest {
                id: "com.nexus.storage".to_string(),
                name: "Storage Engine".to_string(),
                version: "0.1.0".to_string(),
                description: "Forge file indexing and knowledge graph storage".to_string(),
                capabilities: vec!["vault:read".to_string(), "vault:write".to_string()],
                enabled: true,
            },
            PluginManifest {
                id: "com.nexus.ai".to_string(),
                name: "AI Prompt Engine".to_string(),
                version: "0.1.0".to_string(),
                description: "LLM prompt routing and vector RAG search".to_string(),
                capabilities: vec!["ai:prompt".to_string(), "ai:embed".to_string()],
                enabled: true,
            },
            PluginManifest {
                id: "com.nexus.remindme".to_string(),
                name: "RemindMe Integration".to_string(),
                version: "0.1.0".to_string(),
                description: "Personal memory and task reminder bridge".to_string(),
                capabilities: vec!["remindme:sync".to_string()],
                enabled: true,
            },
        ];

        let mut map = self.plugins.lock().unwrap();
        for p in builtins {
            map.insert(p.id.clone(), p);
        }
    }

    pub fn list_plugins(&self) -> Vec<PluginManifest> {
        let map = self.plugins.lock().unwrap();
        let mut list: Vec<PluginManifest> = map.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub fn enable_plugin(&self, id: &str) -> Result<bool, String> {
        let mut map = self.plugins.lock().unwrap();
        if let Some(p) = map.get_mut(id) {
            p.enabled = true;
            Ok(true)
        } else {
            Err(format!("Plugin '{}' not found", id))
        }
    }

    pub fn disable_plugin(&self, id: &str) -> Result<bool, String> {
        let mut map = self.plugins.lock().unwrap();
        if let Some(p) = map.get_mut(id) {
            p.enabled = false;
            Ok(false)
        } else {
            Err(format!("Plugin '{}' not found", id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::new();
        let plugins = manager.list_plugins();
        assert!(plugins.len() >= 3);

        assert!(manager.disable_plugin("com.nexus.ai").is_ok());
        let updated = manager.list_plugins();
        let ai = updated.iter().find(|p| p.id == "com.nexus.ai").unwrap();
        assert!(!ai.enabled);
    }
}

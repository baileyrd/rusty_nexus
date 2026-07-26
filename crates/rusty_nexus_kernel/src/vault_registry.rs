//! Multi-vault registry manager tracking active Nexus forge directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Registered vault entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultEntry {
    pub name: String,
    pub path: PathBuf,
    pub active: bool,
}

/// Central manager tracking registered forge vaults.
pub struct VaultRegistry {
    vaults: HashMap<String, VaultEntry>,
}

impl Default for VaultRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultRegistry {
    pub fn new() -> Self {
        Self {
            vaults: HashMap::new(),
        }
    }

    pub fn register_vault(&mut self, name: &str, path: &Path) {
        let entry = VaultEntry {
            name: name.to_string(),
            path: path.to_path_buf(),
            active: false,
        };
        self.vaults.insert(name.to_string(), entry);
    }

    pub fn list_vaults(&self) -> Vec<VaultEntry> {
        let mut list: Vec<VaultEntry> = self.vaults.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_registry() {
        let mut registry = VaultRegistry::new();
        registry.register_vault("Personal", Path::new("c:/dev/vault_a"));
        registry.register_vault("Work", Path::new("c:/dev/vault_b"));

        let list = registry.list_vaults();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Personal");
    }
}

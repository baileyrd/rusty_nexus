//! Path security validation and audit logging for `rusty_nexus`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Path validator enforcing forge boundary security.
pub struct ForgePathValidator {
    forge_root: PathBuf,
}

impl ForgePathValidator {
    pub fn new(forge_root: &Path) -> Self {
        Self {
            forge_root: forge_root.to_path_buf(),
        }
    }

    /// Validate that `rel_path` resolves strictly inside `forge_root`.
    pub fn validate_path(&self, rel_path: &str) -> Result<PathBuf, String> {
        let trimmed = rel_path.trim();
        if trimmed.starts_with('/') || trimmed.starts_with('\\') || (trimmed.len() > 1 && trimmed.as_bytes()[1] == b':') {
            return Err("Absolute paths are not allowed inside forge".to_string());
        }

        let mut components = Vec::new();
        for comp in Path::new(trimmed).components() {
            match comp {
                std::path::Component::Normal(c) => components.push(c),
                std::path::Component::ParentDir => {
                    if components.pop().is_none() {
                        return Err("Path traversal (..) outside forge root rejected".to_string());
                    }
                }
                std::path::Component::CurDir => {}
                _ => return Err("Invalid path component".to_string()),
            }
        }

        let mut abs = self.forge_root.clone();
        for c in components {
            abs.push(c);
        }

        Ok(abs)
    }
}

/// Audit logger recording forge access and modifications to `.forge/audit.log`.
pub struct AuditLogger {
    log_file_path: PathBuf,
}

impl AuditLogger {
    pub fn new(forge_root: &Path) -> Self {
        let log_file_path = forge_root.join(".forge").join("audit.log");
        Self { log_file_path }
    }

    /// Record an operation in the audit log.
    pub fn log_action(&self, action: &str, target: &str, status: &str) {
        if let Some(parent) = self.log_file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let timestamp = std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs();
        let line = format!(
            "{{\"timestamp\":{},\"action\":\"{}\",\"target\":\"{}\",\"status\":\"{}\"}}\n",
            timestamp,
            action.replace('"', "\\\""),
            target.replace('"', "\\\""),
            status
        );

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_validator_security() {
        let temp_dir = std::env::temp_dir().join(format!("sec_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let validator = ForgePathValidator::new(&temp_dir);

        // Valid relative paths
        assert!(validator.validate_path("notes/guide.md").is_ok());
        assert!(validator.validate_path("notes/sub/doc.md").is_ok());

        // Invalid path traversal attempts
        assert!(validator.validate_path("../outside.txt").is_err());
        assert!(validator.validate_path("notes/../../outside.txt").is_err());
        assert!(validator.validate_path("/etc/passwd").is_err());
        assert!(validator.validate_path("C:\\Windows\\System32").is_err());
    }
}

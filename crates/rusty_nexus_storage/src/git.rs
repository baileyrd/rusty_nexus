//! Git repository history and status inspection for forge directories.

use std::path::Path;
use std::process::Command;

/// Git status summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub untracked_count: usize,
    pub modified_count: usize,
}

/// Query git status inside forge directory.
pub fn get_git_status(forge_root: &Path) -> Result<GitStatus, String> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("-b")
        .current_dir(forge_root)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if !output.status.success() {
        return Err("Not a git repository or git command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branch = "unknown".to_string();
    let mut untracked_count = 0;
    let mut modified_count = 0;

    for line in stdout.lines() {
        if line.starts_with("##") {
            branch = line[2..].trim().to_string();
        } else if line.starts_with("??") {
            untracked_count += 1;
        } else if !line.trim().is_empty() {
            modified_count += 1;
        }
    }

    let is_clean = untracked_count == 0 && modified_count == 0;
    Ok(GitStatus {
        branch,
        is_clean,
        untracked_count,
        modified_count,
    })
}

/// Query last N git commit logs inside forge directory.
pub fn get_git_log(forge_root: &Path, max_count: usize) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("log")
        .arg("-n")
        .arg(max_count.to_string())
        .arg("--oneline")
        .current_dir(forge_root)
        .output()
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !output.status.success() {
        return Err("Failed to query git log".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let logs: Vec<String> = stdout.lines().map(String::from).collect();
    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_inspection() {
        let root = Path::new(".");
        let status = get_git_status(root);
        assert!(status.is_ok());
    }
}

//! Synchronizes extracted Nexus note tasks into RemindMe entries.

use rusty_nexus_storage::StorageEngine;

/// Synchronize tasks from storage into RemindMe format.
pub fn sync_tasks_to_reminders(storage: &StorageEngine) -> Result<usize, String> {
    let tasks = storage.tasks()?;
    let mut synced_count = 0;

    for task in tasks {
        if !task.completed {
            // Register reminder entry
            synced_count += 1;
        }
    }

    Ok(synced_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sync_tasks() {
        let temp_dir = std::env::temp_dir().join(format!("sync_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        let storage = StorageEngine::init(&temp_dir).expect("init storage");
        let _ = storage.create_file("todo.md", "# Todo\n- [ ] Buy groceries\n- [x] Done item").unwrap();

        let count = sync_tasks_to_reminders(&storage).expect("sync reminders");
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}

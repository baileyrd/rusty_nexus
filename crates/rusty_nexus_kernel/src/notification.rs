//! Terminal and system notification manager for `rusty_nexus`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Severity level of a system notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

/// System notification message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp_sec: u64,
}

/// Central notification manager queue.
pub struct NotificationManager {
    notifications: Arc<Mutex<VecDeque<Notification>>>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Post a notification to the manager.
    pub fn post(&self, title: &str, message: &str, severity: NotificationSeverity) {
        let ts = std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs();
        let notif = Notification {
            id: format!("notif_{}", ts),
            title: title.to_string(),
            message: message.to_string(),
            severity,
            timestamp_sec: ts,
        };

        let mut queue = self.notifications.lock().unwrap();
        queue.push_back(notif);
        if queue.len() > 50 {
            queue.pop_front();
        }
    }

    /// Retrieve all unread/active notifications.
    pub fn get_notifications(&self) -> Vec<Notification> {
        let queue = self.notifications.lock().unwrap();
        queue.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_manager() {
        let manager = NotificationManager::new();
        manager.post("Storage", "File created", NotificationSeverity::Info);
        manager.post("Graph", "Broken wikilink found", NotificationSeverity::Warning);

        let notifs = manager.get_notifications();
        assert_eq!(notifs.len(), 2);
        assert_eq!(notifs[0].title, "Storage");
        assert_eq!(notifs[1].severity, NotificationSeverity::Warning);
    }
}

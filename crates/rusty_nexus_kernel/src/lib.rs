//! Microkernel for `rusty_nexus`.
//!
//! Provides the central event bus, capability verification, path security validation, and IPC message dispatching.

pub mod crdt;
pub mod notification;
pub mod plugin;
pub mod security;
pub mod workflow;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rusty_nexus_types::IpcMessage;

pub type EventHandler = Box<dyn Fn(&IpcMessage) + Send + Sync + 'static>;

/// Central event bus and dispatcher for Nexus core subsystems.
pub struct Kernel {
    listeners: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    capabilities: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

impl Kernel {
    /// Create a new Kernel instance.
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            capabilities: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a capability grant for a subsystem/plugin.
    pub fn grant_capability(&self, subsystem: &str, capability: &str) {
        let mut caps = self.capabilities.lock().unwrap();
        caps.entry(subsystem.to_string())
            .or_default()
            .push(capability.to_string());
    }

    /// Check whether a subsystem holds a specific capability.
    pub fn has_capability(&self, subsystem: &str, capability: &str) -> bool {
        let caps = self.capabilities.lock().unwrap();
        if let Some(granted) = caps.get(subsystem) {
            granted.iter().any(|c| c == capability)
        } else {
            false
        }
    }

    /// Subscribe to events by name on the kernel event bus.
    pub fn subscribe<F>(&self, event_name: &str, handler: F)
    where
        F: Fn(&IpcMessage) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event_name.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// Publish an event to all registered subscribers.
    pub fn publish(&self, message: IpcMessage) {
        let listeners = self.listeners.lock().unwrap();
        if let Some(handlers) = listeners.get(&message.event_name) {
            for handler in handlers {
                handler(&message);
            }
        }
    }
}

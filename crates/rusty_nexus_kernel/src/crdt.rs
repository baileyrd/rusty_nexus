//! CRDT (Conflict-free Replicated Data Type) primitives for real-time note editing.

/// Last-Write-Wins (LWW) Register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LwwRegister<T> {
    pub value: T,
    pub timestamp_ms: u64,
    pub peer_id: String,
}

impl<T: Clone> LwwRegister<T> {
    pub fn new(value: T, timestamp_ms: u64, peer_id: String) -> Self {
        Self {
            value,
            timestamp_ms,
            peer_id,
        }
    }

    /// Merge incoming register value using Last-Write-Wins semantics.
    pub fn merge(&mut self, incoming: Self) -> bool {
        if incoming.timestamp_ms > self.timestamp_ms
            || (incoming.timestamp_ms == self.timestamp_ms && incoming.peer_id > self.peer_id)
        {
            self.value = incoming.value;
            self.timestamp_ms = incoming.timestamp_ms;
            self.peer_id = incoming.peer_id;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_lww_merge() {
        let mut r1 = LwwRegister::new("Original", 100, "peer_a".to_string());
        let r2 = LwwRegister::new("Updated", 200, "peer_b".to_string());

        let merged = r1.merge(r2);
        assert!(merged);
        assert_eq!(r1.value, "Updated");

        // Outdated update rejected
        let r3 = LwwRegister::new("Outdated", 50, "peer_c".to_string());
        assert!(!r1.merge(r3));
        assert_eq!(r1.value, "Updated");
    }
}

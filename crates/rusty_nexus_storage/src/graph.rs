//! In-memory Knowledge Graph for note linkages, tags, and unresolved links.

use std::collections::{HashMap, HashSet};
use rusty_nexus_types::GraphNode;

/// Graph summary metrics.
#[derive(Debug, Clone, Default)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub unresolved_count: usize,
}

/// Knowledge graph tracking links and nodes across the forge.
#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraph {
    nodes: HashMap<String, GraphNode>,
    outgoing: HashMap<String, HashSet<String>>,
    incoming: HashMap<String, HashSet<String>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a node in the graph.
    pub fn add_node(&mut self, path: String, title: String, node_type: String) {
        self.nodes.insert(
            path.clone(),
            GraphNode {
                path,
                title,
                node_type,
            },
        );
    }

    /// Remove a node and its associated links.
    pub fn remove_node(&mut self, path: &str) {
        self.nodes.remove(path);
        if let Some(targets) = self.outgoing.remove(path) {
            for target in targets {
                if let Some(sources) = self.incoming.get_mut(&target) {
                    sources.remove(path);
                }
            }
        }
        if let Some(sources) = self.incoming.remove(path) {
            for source in sources {
                if let Some(targets) = self.outgoing.get_mut(&source) {
                    targets.remove(path);
                }
            }
        }
    }

    /// Add a directional edge/link from source to target.
    pub fn add_edge(&mut self, source: String, target: String) {
        self.outgoing
            .entry(source.clone())
            .or_default()
            .insert(target.clone());
        self.incoming.entry(target).or_default().insert(source);
    }

    /// Get overall graph statistics.
    pub fn stats(&self) -> GraphStats {
        let node_count = self.nodes.len();
        let mut edge_count = 0;
        for targets in self.outgoing.values() {
            edge_count += targets.len();
        }
        let unresolved_count = self.unresolved_links().len();

        GraphStats {
            node_count,
            edge_count,
            unresolved_count,
        }
    }

    /// Get all target links that do not map to an existing note in the graph.
    pub fn unresolved_links(&self) -> Vec<String> {
        let mut unresolved = HashSet::new();
        for (_source, targets) in &self.outgoing {
            for target in targets {
                // Check if target matches any node path or node title
                let exists = self.nodes.values().any(|n| n.path == *target || n.title == *target);
                if !exists {
                    unresolved.insert(target.clone());
                }
            }
        }
        let mut list: Vec<String> = unresolved.into_iter().collect();
        list.sort();
        list
    }

    /// Find outgoing/incoming neighbors within a given depth.
    pub fn neighbors(&self, path: &str, depth: usize) -> Vec<GraphNode> {
        let mut visited = HashSet::new();
        let mut queue = vec![(path.to_string(), 0)];
        visited.insert(path.to_string());

        let mut result = Vec::new();

        while let Some((curr, curr_depth)) = queue.pop() {
            if curr != path {
                if let Some(node) = self.nodes.get(&curr) {
                    result.push(node.clone());
                }
            }

            if curr_depth < depth {
                if let Some(out_targets) = self.outgoing.get(&curr) {
                    for target in out_targets {
                        if !visited.contains(target) {
                            visited.insert(target.clone());
                            queue.push((target.clone(), curr_depth + 1));
                        }
                    }
                }
                if let Some(in_sources) = self.incoming.get(&curr) {
                    for source in in_sources {
                        if !visited.contains(source) {
                            visited.insert(source.clone());
                            queue.push((source.clone(), curr_depth + 1));
                        }
                    }
                }
            }
        }

        result
    }

    /// Get backlinks (incoming links) for a note.
    pub fn backlinks(&self, path: &str) -> Vec<String> {
        let mut sources_list = Vec::new();
        if let Some(sources) = self.incoming.get(path) {
            sources_list = sources.iter().cloned().collect();
            sources_list.sort();
        } else {
            // Check by title match
            if let Some(node) = self.nodes.get(path) {
                if let Some(sources) = self.incoming.get(&node.title) {
                    sources_list = sources.iter().cloned().collect();
                    sources_list.sort();
                }
            }
        }
        sources_list
    }
}

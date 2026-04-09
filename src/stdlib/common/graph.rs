use crate::stdlib::common::Result;
use bund_blobstore::{BlobStore, DataDistributionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================
// GRAPH STORAGE USING THE MANAGER
// ============================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    id: String,
    label: String,
    properties: HashMap<String, String>,
}

// Simple graph edge structure with Serde support
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    from: String,
    to: String,
    weight: f64,
    relationship: String,
}

// Graph storage wrapper that uses the same manager
pub struct ManagedGraphStore {
    pub _manager: Arc<DataDistributionManager>,
    pub store: Mutex<BlobStore>,
}

impl ManagedGraphStore {
    pub fn new(manager: Arc<DataDistributionManager>, path: &str) -> Result<Self> {
        let store = BlobStore::open(path)?;
        Ok(Self {
            _manager: manager,
            store: Mutex::new(store),
        })
    }

    pub fn add_node(&self, node: GraphNode) -> Result<()> {
        let key = format!("node:{}", node.id);
        let data = serde_json::to_vec(&node)?;
        Ok(self
            .store
            .lock()
            .unwrap()
            .put(&key, &data, Some("graph_nodes"))?)
    }

    pub fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        let key = format!("node:{}", id);
        if let Some(data) = self.store.lock().unwrap().get(&key)? {
            let node: GraphNode = serde_json::from_slice(&data)?;
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    pub fn add_edge(&self, edge: GraphEdge) -> Result<()> {
        let key = format!("edge:{}:{}", edge.from, edge.to);
        let data = serde_json::to_vec(&edge)?;
        Ok(self
            .store
            .lock()
            .unwrap()
            .put(&key, &data, Some("graph_edges"))?)
    }

    pub fn get_edges_from(&self, from: &str) -> Result<Vec<GraphEdge>> {
        let prefix = format!("edge:{}:", from);
        let all_keys = self.store.lock().unwrap().list_keys()?;
        let mut edges: Vec<GraphEdge> = Vec::new();

        for key in all_keys {
            if key.starts_with(&prefix) {
                if let Some(data) = self.store.lock().unwrap().get(&key)? {
                    if let Ok(edge) = serde_json::from_slice(&data) {
                        edges.push(edge);
                    }
                }
            }
        }

        Ok(edges)
    }

    pub fn find_shortest_path(&self, start: &str, end: &str) -> Result<Option<Vec<String>>> {
        let mut visited = HashMap::new();
        let mut queue = vec![(start.to_string(), vec![start.to_string()])];
        visited.insert(start.to_string(), true);

        while !queue.is_empty() {
            let (current, path) = queue.remove(0);

            if current == end {
                return Ok(Some(path));
            }

            let edges = self.get_edges_from(&current)?;
            for edge in edges {
                if !visited.contains_key(&edge.to) {
                    visited.insert(edge.to.clone(), true);
                    let mut new_path = path.clone();
                    new_path.push(edge.to.clone());
                    queue.push((edge.to, new_path));
                }
            }
        }

        Ok(None)
    }
}

pub mod graph;
pub mod logs;

// Custom error type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use graph::{GraphEdge, GraphNode, ManagedGraphStore};

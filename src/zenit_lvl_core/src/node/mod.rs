use std::fmt::Debug;

pub mod builder;
pub mod name;
pub mod parser;
pub mod reader;

pub use builder::NodeBuilder;
pub use name::NodeName;
pub use parser::NodeParser;
pub use reader::NodeReadExt;

/// Represents a node in a level file.
#[derive(Debug, Clone)]
pub struct LevelNode {
    /// 4-byte name identifier
    pub name: NodeName,
    /// Offset of the node's payload in its file
    pub payload_offset: u64,
    /// Size of the node's payload
    pub payload_size: u32,
}

//! Automatic structure parsing solutions through painful traits, macros and externally defined
//! derive macros.

pub mod macros;
pub mod packed;
pub mod parse;

pub use crate::define_node_type;
pub use parse::NodeParser;
pub use packed::PackedParser;

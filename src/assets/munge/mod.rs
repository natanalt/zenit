use std::io::{self, Read, Seek, SeekFrom};

mod name;
pub mod parser;

pub use name::MungeName;

#[derive(Debug, Clone, Copy)]
pub struct MungeNode {
    pub name: MungeName,
    pub offset: u64,
    pub length: u32,
}

impl MungeNode {
    
}

#[derive(Debug, Clone)]
pub struct MungeTreeNode {
    pub node: MungeNode,
    pub children: Vec<MungeTreeNode>,
}

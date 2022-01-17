
mod name;
pub mod parser;

pub use name::MungeName;

#[derive(Debug, Clone, Copy)]
pub struct MungeNode {
    pub name: MungeName,
    pub offset: u64,
    pub length: u32,
}

#[derive(Debug, Clone)]
pub struct MungeTreeNode {
    pub node: MungeNode,
    pub children: Vec<MungeTreeNode>,
}

impl MungeTreeNode {
    pub fn find(&self, name: MungeName) -> Option<&MungeTreeNode> {
        self.children.iter().find(|x| x.node.name == name)
    }
}

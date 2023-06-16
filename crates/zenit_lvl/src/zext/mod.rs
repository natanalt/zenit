//! Zenit Engine extensions to the level format

use crate::node::NodeData;
use std::ffi::CString;

#[derive(Debug, Clone, NodeData)]
pub struct LevelWgslShader {
    #[node("NAME")]
    pub name: CString,
    #[node("CODE")]
    pub code: CString,
}

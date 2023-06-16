use crate::node::{LazyData, NodeData};
use std::ffi::CString;

#[derive(Debug, Clone, NodeData)]
pub struct LevelScript {
    #[node("NAME")]
    pub name: CString,
    #[node("INFO")]
    pub info: u8,
    #[node("BODY")]
    pub data: LazyData<Vec<u8>>,
}

use std::ffi::CString;
use crate::LazyData;
use zenit_proc::FromNode;

#[derive(Debug, Clone, FromNode)]
pub struct LevelScript {
    #[node("NAME")]
    pub name: CString,
    #[node("INFO")]
    pub info: u8,
    #[node("BODY")]
    pub data: LazyData<Vec<u8>>,
}

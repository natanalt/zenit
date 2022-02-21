use std::ffi::CString;
use zenit_proc::{ext_repr, NodeParser, PackedParser};

pub use zenit_lvl_core::*;

#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum FormatKind {
    A,
}
#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum TextureKind {
    A,
}


#[derive(Debug, Clone, NodeParser)]
pub struct LevelData {
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,
}

#[derive(Debug, Clone, NodeParser)]
pub struct LevelScript {
    #[node("NAME")]
    pub name: CString,
    #[node("INFO")]
    pub info: u8,
    //#[node("BODY")]
    //pub data: LazyData<u8>,
}

#[derive(Debug, Clone, NodeParser)]
pub struct LevelTexture {
    #[node("NAME")]
    pub name: CString,
    #[nodes("FMT_")]
    pub formats: Vec<TextureFormat>,
}

#[derive(Debug, Clone, NodeParser)]
pub struct TextureFormat {
    #[node("INFO")]
    pub info: TextureInfo,
    #[nodes("FACE")]
    pub faces: Vec<TextureFace>,
}

#[derive(Debug, Clone, NodeParser)]
pub struct TextureFace {
    #[nodes("LVL_")]
    pub mipmaps: Vec<TextureMipmap>,
}

#[derive(Debug, Clone, NodeParser)]
pub struct TextureMipmap {
    //#[node("INFO")]
    //pub info: MipmapInfo,
    //#[node("BODY")]
    //pub body: LazyData<u8>,
}

#[derive(Debug, Clone, PackedParser)]
pub struct TextureInfo {
    #[from(u32)]
    pub format: FormatKind,
    pub width: u16,
    pub height: u16,
    pub unknown: u16,
    pub mipmaps: u16,
    #[from(u32)]
    pub kind: TextureKind,
}


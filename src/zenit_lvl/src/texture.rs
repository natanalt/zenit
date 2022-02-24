use std::ffi::CString;
use zenit_lvl_core::LazyData;
use zenit_proc::{ext_repr, FromNode, PackedParser};
use zenit_utils::string_as_u32;

#[derive(Debug, Clone, FromNode)]
pub struct LevelTexture {
    #[node("NAME")]
    pub name: CString,
    #[nodes("FMT_")]
    pub formats: Vec<TextureFormat>,
}

#[derive(Debug, Clone, FromNode)]
pub struct TextureFormat {
    #[node("INFO")]
    pub info: TextureInfo,
    #[nodes("FACE")]
    pub faces: Vec<TextureFace>,
}

#[derive(Debug, Clone, FromNode)]
pub struct TextureFace {
    #[nodes("LVL_")]
    pub mipmaps: Vec<TextureMipmap>,
}

#[derive(Debug, Clone, FromNode)]
pub struct TextureMipmap {
    //#[node("INFO")]
    //pub info: MipmapInfo,
    #[node("BODY")]
    pub body: LazyData<u8>,
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

#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum FormatKind {
    DXT1 = string_as_u32("DXT1"),
    DXT3 = string_as_u32("DXT3"),
    A8R8G8B8 = 0x15,
    R5G6B5 = 0x17,
    A1R5G5B5 = 0x19,
    A4R4G4B4 = 0x1a,
    A8 = 0x1c,
    L8 = 0x32,
    A8L8 = 0x33,
    A4L4 = 0x34,
    V8U8 = 0x3c,
}

#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum TextureKind {
    Normal = 1,
    Cubemap = 2,
}

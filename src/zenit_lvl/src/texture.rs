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
    pub info: TextureFormatInfo,
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
    #[node("INFO")]
    pub info: MipmapInfo,
    #[node("BODY")]
    pub body: LazyData<Vec<u8>>,
}

#[derive(Debug, Clone, PackedParser)]
pub struct MipmapInfo {
    pub mip_level: u32,
    pub body_size: u32,
}

#[derive(Debug, Clone, PackedParser)]
pub struct TextureFormatInfo {
    #[from(u32)]
    pub format: FormatKind,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub mipmaps: u16,
    #[from(u32)]
    pub kind: TextureKind,
}

#[ext_repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatKind {
    // List seems to be based on D3D9
    // The following are the only formats I happened to see used by the munge tools.
    
    /// Compressed texture
    DXT1 = string_as_u32("DXT1"),
    /// Compressed texture
    DXT3 = string_as_u32("DXT3"),
    /// RGBA, u8 value per channel
    A8R8G8B8 = 0x15,
    /// 16-bit RGB
    R5G6B5 = 0x17,
    /// 16-bit RGB with a single alpha bit
    A1R5G5B5 = 0x19,
    /// 16-bit RGBA
    A4R4G4B4 = 0x1a,
    /// 8-bit alpha only
    A8 = 0x1c,
    /// 8-bit luminance channel only
    /// (D3D9 included custom lighting stuff)
    L8 = 0x32,
    /// 8-bit alpha + 8-bit luminance
    A8L8 = 0x33,
    /// 4-bit alpha + 4-bit luminance
    A4L4 = 0x34,
    /// 2D vector map
    V8U8 = 0x3c,
}

impl FormatKind {
    pub fn is_compressed(self) -> bool {
        match self {
            FormatKind::DXT1 | FormatKind::DXT3 => true,
            FormatKind::A8R8G8B8
            | FormatKind::R5G6B5
            | FormatKind::A1R5G5B5
            | FormatKind::A4R4G4B4
            | FormatKind::A8
            | FormatKind::L8
            | FormatKind::A8L8
            | FormatKind::A4L4
            | FormatKind::V8U8 => false,
        }
    }
}

#[ext_repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureKind {
    Normal = 1,
    Cubemap = 2,
}

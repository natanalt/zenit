use crate::LazyData;
use std::ffi::CString;
use zenit_proc::{ext_repr, FromNode, PackedParser};
use zenit_utils::string_as_u32;

#[derive(Debug, Clone, FromNode)]
pub struct LevelTexture {
    #[node("NAME")]
    pub name: CString,
    #[nodes("FMT_")]
    pub formats: Vec<LevelTextureFormat>,
}

#[derive(Debug, Clone, FromNode)]
pub struct LevelTextureFormat {
    #[node("INFO")]
    pub info: LevelTextureFormatInfo,
    #[nodes("FACE")]
    pub faces: Vec<LevelTextureFace>,
}

#[derive(Debug, Clone, FromNode)]
pub struct LevelTextureFace {
    #[nodes("LVL_")]
    pub mipmaps: Vec<LevelTextureMipmap>,
}

#[derive(Debug, Clone, FromNode)]
pub struct LevelTextureMipmap {
    #[node("INFO")]
    pub info: LevelTextureMipmapInfo,
    #[node("BODY")]
    pub body: LazyData<Vec<u8>>,
}

#[derive(Debug, Clone, PackedParser)]
pub struct LevelTextureMipmapInfo {
    pub mip_level: u32,
    pub body_size: u32,
}

#[derive(Debug, Clone, PackedParser)]
pub struct LevelTextureFormatInfo {
    #[from(u32)]
    pub format: LevelTextureFormatKind,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub mipmaps: u16,
    #[from(u32)]
    pub kind: LevelTextureKind,
}

#[ext_repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelTextureFormatKind {
    // The formats are derived from texture formats supported by D3D9
    // All entries in this enum are ones I found to be used by texture munge
    // tools, despite Direct3D support waaaay more formats than that
    /// DXT1 (also known as BC1 in wgpu) compressed texture.
    DXT1 = string_as_u32("DXT1"),
    /// DXT3 (also known as BC2 in wgpu) compressed texture.
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
    /// (D3D9 included built-in lighting stuff, cause early 2000s APIs)
    L8 = 0x32,
    /// 8-bit alpha + 8-bit luminance
    A8L8 = 0x33,
    /// 4-bit alpha + 4-bit luminance
    A4L4 = 0x34,
    /// 2D vector map
    V8U8 = 0x3c,
}

impl LevelTextureFormatKind {
    pub fn channel_count(self) -> u32 {
        match self {
            LevelTextureFormatKind::DXT1 => 4,
            LevelTextureFormatKind::DXT3 => 4,
            LevelTextureFormatKind::A8R8G8B8 => 4,
            LevelTextureFormatKind::R5G6B5 => 3,
            LevelTextureFormatKind::A1R5G5B5 => 4,
            LevelTextureFormatKind::A4R4G4B4 => 4,
            LevelTextureFormatKind::A8 => 1,
            LevelTextureFormatKind::L8 => 1,
            LevelTextureFormatKind::A8L8 => 2,
            LevelTextureFormatKind::A4L4 => 2,
            LevelTextureFormatKind::V8U8 => 2,
        }
    }

    pub fn is_compressed(self) -> bool {
        match self {
            LevelTextureFormatKind::DXT1 | LevelTextureFormatKind::DXT3 => true,
            LevelTextureFormatKind::A8R8G8B8
            | LevelTextureFormatKind::R5G6B5
            | LevelTextureFormatKind::A1R5G5B5
            | LevelTextureFormatKind::A4R4G4B4
            | LevelTextureFormatKind::A8
            | LevelTextureFormatKind::L8
            | LevelTextureFormatKind::A8L8
            | LevelTextureFormatKind::A4L4
            | LevelTextureFormatKind::V8U8 => false,
        }
    }
}

#[ext_repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelTextureKind {
    Normal = 1,
    Cubemap = 2,
}

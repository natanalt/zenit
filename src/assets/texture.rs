use bevy::{reflect::TypeUuid, math::IVec2};
use num_derive::FromPrimitive;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "8aa0f93e-8037-4742-8458-04ab5154e133"]
pub struct MungeTexture {
    pub name: String,
    pub formats: Vec<TextureFormat>,
}

#[derive(Debug, Clone)]
pub struct TextureFormat {
    pub size: IVec2,
    pub format: FormatKind,
    pub mysterious_flags: u16,
    pub mipmaps: TextureMipmaps,
}

#[derive(Debug, Clone)]
pub enum TextureMipmaps {
    Normal(Vec<MipmapLevel>),
    Cubemap([Vec<MipmapLevel>; 6]),
}

#[derive(Debug, Clone)]
pub struct MipmapLevel {
    pub level: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
#[repr(u32)]
pub enum FormatKind {
    DXT1 = 0x31_54_58_44,
    DXT3 = 0x33_54_58_44,
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

#[derive(Debug, Clone, Copy, FromPrimitive)]
#[repr(u32)]
pub enum TextureKind {
    Normal = 1,
    Cubemap = 2,
}

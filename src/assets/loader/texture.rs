use std::io::Cursor;
use crate::{
    assets::munge::{MungeName, MungeTreeNode},
    AnyResult,
};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext},
    prelude::*,
    reflect::TypeUuid,
};

pub struct MungeTextureLoader;

impl AssetLoader for MungeTextureLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, AnyResult<()>> {
        Box::pin(async move {
            let mut cursor = Cursor::new(bytes);
            let root = MungeTreeNode::parse(&mut cursor, None)?;

            let name = root
                .find(MungeName::from_literal("NAME"))
                .ok_or(anyhow::anyhow!("Invalid texture node"))?
                .node
                .read_string(&mut cursor)?;

            todo!();

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ztexture"]
    }
}

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
    pub kind: TextureKind,
    pub mysterious_flags: u16,
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TextureKind {
    Normal = 1,
    Cubemap = 2,
}

use crate::{
    assets::{
        munge::{MungeName, MungeTreeNode},
        texture::{
            FormatKind, MipmapLevel, MungeTexture, TextureFormat, TextureKind, TextureMipmaps,
        },
    },
    AnyResult,
};
use anyhow::anyhow;
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
};
use byteorder::{ReadBytesExt, LE};
use num_traits::FromPrimitive;
use std::io::Cursor;

#[derive(Debug, Clone, Copy, Default)]
pub struct MungeTextureLoaderPlugin;

impl Plugin for MungeTextureLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<MungeTexture>()
            .init_asset_loader::<MungeTextureLoader>();
    }
}

#[derive(Debug, Clone, Copy, Default)]
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
                .ok_or(anyhow!("Invalid texture node (no name)"))?
                .node
                .read_string(&mut cursor)?;

            info!("Loading texture `{}`", &name);

            let formats = root
                .children
                .iter()
                .filter(|x| x.node.name == MungeName::from_literal("FMT_"))
                .map(|fmt_node| {
                    let mut info_reader = fmt_node
                        .find(MungeName::from_literal("INFO"))
                        .ok_or(anyhow!("No format info node"))?
                        .node
                        .read_into_cursor(&mut cursor)?;

                    let format = FormatKind::from_u32(info_reader.read_u32::<LE>()?)
                        .ok_or(anyhow!("Invalid texture format"))?;
                    let size = IVec2::new(
                        info_reader.read_u16::<LE>()? as i32,
                        info_reader.read_u16::<LE>()? as i32,
                    );
                    let mysterious_flags = info_reader.read_u16::<LE>()?;
                    let _mipmap_amount = info_reader.read_u16::<LE>()?;

                    let texture_kind = TextureKind::from_u32(info_reader.read_u32::<LE>()?)
                        .ok_or(anyhow!("Invalid texture kind"))?;
                    let mipmaps = match texture_kind {
                        TextureKind::Normal => TextureMipmaps::Normal(
                            fmt_node
                                .find(MungeName::from_literal("FACE"))
                                .ok_or(anyhow!("Invalid texture format (no single face)"))?
                                .children
                                .iter()
                                .filter(|x| x.node.name == MungeName::from_literal("LVL_"))
                                .map(|node| mipmap_loader(&mut cursor, node))
                                .collect::<AnyResult<Vec<MipmapLevel>>>()?,
                        ),
                        TextureKind::Cubemap => TextureMipmaps::Cubemap(
                            fmt_node
                                .children
                                .iter()
                                .filter(|x| x.node.name == MungeName::from_literal("FACE"))
                                .map(|face_node| {
                                    Ok(face_node
                                        .children
                                        .iter()
                                        .filter(|x| x.node.name == MungeName::from_literal("LVL_"))
                                        .map(|node| mipmap_loader(&mut cursor, node))
                                        .collect::<AnyResult<Vec<MipmapLevel>>>()?)
                                })
                                .collect::<AnyResult<Vec<Vec<MipmapLevel>>>>()?
                                .try_into()
                                .unwrap(),
                        ),
                    };

                    Ok(TextureFormat {
                        size,
                        format,
                        mysterious_flags,
                        mipmaps,
                    })
                })
                .collect::<AnyResult<Vec<TextureFormat>>>()?;

            load_context.set_default_asset(LoadedAsset::new(MungeTexture { name, formats }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ztexture"]
    }
}

fn mipmap_loader(c: &mut Cursor<&[u8]>, node: &MungeTreeNode) -> AnyResult<MipmapLevel> {
    let level = node
        .find(MungeName::from_literal("INFO"))
        .ok_or(anyhow!("Invalid texture mipmap (no info)"))?
        .node
        .read_into_cursor(c)?
        .read_u32::<LE>()?;

    let data = node
        .find(MungeName::from_literal("BODY"))
        .ok_or(anyhow!("Invalid texture mipmap (no body)"))?
        .node
        .read_contents(c)?;

    Ok(MipmapLevel { level, data })
}

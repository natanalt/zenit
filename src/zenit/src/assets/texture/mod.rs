use self::convert::convert_texture;
use crate::render::{
    texture::{Texture2D, TextureSource},
    RenderContext,
};
use anyhow::{anyhow, bail};
use glam::UVec2;
use std::io::{Read, Seek};
use zenit_lvl::raw::texture::{LevelTexture, LevelTextureKind};
use zenit_utils::AnyResult;

pub mod convert;

pub enum TextureAsset {
    Texture2D(Texture2D),
    TextureCube(()),
}

impl TextureAsset {
    pub fn load<R: Read + Seek>(
        ctx: &RenderContext,
        data: &LevelTexture,
        r: &mut R,
    ) -> AnyResult<Self> {
        if data.formats.is_empty() {
            bail!("No formats?");
        }

        let format = data
            .formats
            .iter()
            .find(|f| !f.info.format.is_compressed())
            .ok_or(anyhow!("Compressed textures are unsupported"))?;

        match format.info.kind {
            LevelTextureKind::Normal => {
                let base_size = UVec2::new(format.info.width as u32, format.info.height as u32);
                let face = format
                    .faces
                    .first()
                    .ok_or(anyhow!("Invalid texture storage"))?;

                let texture = TextureSource::new(
                    ctx,
                    &wgpu::TextureDescriptor {
                        label: data.name.to_str().ok(),
                        size: wgpu::Extent3d {
                            width: base_size.x,
                            height: base_size.y,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: face.mipmaps.len() as u32,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: convert::convert_format(format.info.format)
                            .ok_or(anyhow!("Unsupported texture format (see logs)"))?,
                        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                    },
                )
                .into_2d();

                for mipmap in &face.mipmaps {
                    let body = mipmap.body.read(r)?;

                    let level = mipmap.info.mip_level;
                    let downscale_factor = 2u32.pow(level);
                    let mip_size = base_size / downscale_factor;

                    let formatted = convert_texture(body, mip_size, format.info.format).unwrap();

                    texture.write_to_mip(ctx, level, &formatted);
                }

                Ok(Self::Texture2D(texture))
            }
            LevelTextureKind::Cubemap => bail!("Cubemaps aren't supported yet"),
        }
    }
}

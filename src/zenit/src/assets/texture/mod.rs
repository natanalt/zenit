pub mod convert;

use crate::render::base::texture::Texture2D;
use anyhow::{anyhow, bail};
use std::sync::Arc;
use zenit_lvl::texture::{LevelTexture, TextureKind};
use zenit_utils::AnyResult;

pub struct TextureAsset {
    pub name: String,
    pub gpu_texture: Arc<Texture2D>,
}

impl TextureAsset {
    pub fn from_level(texture: LevelTexture) -> AnyResult<Self> {
        if texture.formats.is_empty() {
            bail!("No formats?");
        }

        let _name = texture.name.to_string_lossy().to_owned();
        let format = texture
            .formats
            .iter()
            .find(|f| !f.info.format.is_compressed())
            .ok_or(anyhow!("Compressed textures are unsupported"))?;
        
        match format.info.kind {
            TextureKind::Normal => todo!(),
            TextureKind::Cubemap => todo!(),
            _ => bail!("Format unsupported: {:?}", format.info.kind),
        }
    }
}

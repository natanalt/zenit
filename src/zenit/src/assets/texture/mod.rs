use crate::render::base::{texture::Texture2D, context::RenderContext};
use anyhow::{anyhow, bail};
use zenit_lvl::raw::texture::{LevelTexture, LevelTextureKind};
use zenit_utils::AnyResult;

pub mod convert;

pub enum TextureAsset {
    Texture2D(Texture2D),
    TextureCube(()),
}

impl TextureAsset {
    /// Tries to load a texture into the GPU
    pub fn load(texture: LevelTexture, _context: &RenderContext) -> AnyResult<(String, Self)> {
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
            LevelTextureKind::Normal => todo!(),
            LevelTextureKind::Cubemap => todo!(),
        }
    }
}

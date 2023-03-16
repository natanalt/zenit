use std::sync::Arc;
use crate::render::{TextureResource, TextureHandle, Renderer};
use zenit_lvl::game::texture::LevelTexture;
use zenit_utils::AnyResult;

/// Loads a BF2 texture
pub fn load_level_texture(renderer: &mut Renderer, level: &LevelTexture) -> AnyResult<TextureHandle> {
    todo!()
}

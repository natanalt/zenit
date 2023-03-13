use std::sync::Arc;
use crate::render::{api::Renderer, resources::Texture2D};
use zenit_lvl::game::texture::LevelTexture;
use zenit_utils::AnyResult;

/// Loads a BF2 texture
pub fn load_level_texture(renderer: &Renderer, level: &LevelTexture) -> AnyResult<Arc<Texture2D>> {
    todo!()
}

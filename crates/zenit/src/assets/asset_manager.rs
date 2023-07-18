use std::path::PathBuf;
use super::game_root::GameRoot;
use crate::graphics::{CubemapHandle, Renderer, TextureDescriptor, TextureHandle};
use ahash::AHashMap;
use glam::uvec2;
use wgpu::TextureFormat;

/// The asset manager is responsible for:
///  * loading assets from files
///  * caching loaded assets and allowing them to be accessed via a name
pub struct AssetManager {
    pub game_root: GameRoot,

    pub textures: AHashMap<String, TextureHandle>,
    pub cubemaps: AHashMap<String, CubemapHandle>,

    /// Fallback texture for failed lookups
    pub error_texture: TextureHandle,
}

impl AssetManager {
    pub fn new(game_root: GameRoot, renderer: &mut Renderer) -> Self {
        Self {
            game_root,
            textures: AHashMap::default(),
            cubemaps: AHashMap::default(),

            // Temporary texture, overwritten later
            error_texture: renderer.create_texture(&TextureDescriptor {
                name: String::from("Temporary texture"),
                size: uvec2(1, 1),
                mip_levels: 1,
                format: TextureFormat::Rgba8Unorm,
                unfiltered: true,
                d3d_format: None,
            }),
        }
    }
}

pub struct LevelFile {
    pub label: String,
    pub path: Option<PathBuf>,
}

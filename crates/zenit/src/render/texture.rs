use std::sync::Arc;
use zenit_utils::ArcPoolHandle;
use wgpu::*;
use glam::*;
use zenit_proc::ext_repr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureHandle(pub(super) ArcPoolHandle);

pub use zenit_lvl::game::texture::LevelTextureKind as TextureKind;

/// A 2D texture stored on the GPU.
/// 
/// The texture can be a 1 layered surface, or a 6 layered cubemap. For cubemaps, layers correspond
/// to different faces of the cube (in order: +X, -X, +Y, -Y, +Z, -Z).
pub struct TextureResource {
    pub handle: wgpu::Texture,
    pub kind: TextureKind,
    pub view: Arc<wgpu::TextureView>,
}

#[ext_repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CubemapFace {
    Right = 0,
    Left = 1,
    Up = 2,
    Down = 3,
    Forward = 4,
    Backward = 5,
}

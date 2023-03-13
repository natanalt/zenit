use wgpu::*;
use glam::*;
use zenit_proc::ext_repr;

pub use zenit_lvl::game::texture::LevelTextureKind as TextureKind;

/// A 2D texture stored on the GPU.
/// 
/// The texture can be a 1 layered surface, or a 6 layered cubemap. For cubemaps, layers correspond
/// to different faces of the cube (in order: +X, -X, +Y, -Y, +Z, -Z).
pub struct Texture2D {
    pub handle: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub kind: TextureKind,
}

impl Texture2D {
    /// Returns the 2D size of this texture
    pub fn size(&self) -> UVec2 {
        uvec2(self.handle.width(), self.handle.height())
    }
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

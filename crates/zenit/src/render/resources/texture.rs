use wgpu::*;
use glam::*;
use zenit_proc::ext_repr;

/// A 2D texture stored on the GPU.
pub struct Texture2D {
    pub handle: wgpu::Texture,
    pub view: wgpu::TextureView,
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

/// A 2D cubemap stored on the GPU.
/// 
/// Internally, a cubemap is a 2D texture with 6 layers. Those layers correspond to different
/// faces of the cube (+X, -X, +Y, -Y, +Z, -Z)
pub struct TextureCubemap {
    pub handle: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl TextureCubemap {

}

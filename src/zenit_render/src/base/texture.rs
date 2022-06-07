use std::sync::Arc;
use glam::*;
use super::target::RenderTarget;

/// Wrapper around wgpu textures, containing metadata, and in the future,
/// additional capabilities like asynchronous texture loading
pub struct Texture2D {
    pub texture: wgpu::Texture,
    pub view: Arc<wgpu::TextureView>,
    pub size: IVec2,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
}

impl RenderTarget for Texture2D {
    fn get_current_view(&self) -> Arc<wgpu::TextureView> {
        #[cfg(debug_assertions)]
        if !self.usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            panic!("Attempted to use an illegal texture as render target");
        }
        
        self.view.clone()
    }

    fn get_size(&self) -> IVec2 {
        self.size
    }

    fn get_format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

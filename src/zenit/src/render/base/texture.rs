use super::target::RenderTarget;
use glam::*;
use std::sync::Arc;

/// Wrapper around wgpu textures, containing metadata, and in the future,
/// additional capabilities like asynchronous texture loading
pub struct Texture2D {
    pub texture: wgpu::Texture,
    pub view: Arc<wgpu::TextureView>,
    pub size: IVec2,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
}

impl Texture2D {
    pub fn new(
        device: &wgpu::Device,
        size: IVec2,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.x as _,
                height: size.y as _,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));

        Self {
            texture,
            view,
            size,
            format,
            usage,
        }
    }
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

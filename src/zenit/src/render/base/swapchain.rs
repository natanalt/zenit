use super::target::RenderTarget;
use glam::IVec2;
use std::sync::{Arc, RwLock};

pub struct SwapchainTexture {
    pub surface: wgpu::Surface,
    pub surface_format: wgpu::TextureFormat,
    pub surface_config: RwLock<wgpu::SurfaceConfiguration>,
    current: RwLock<Option<SwapchainCurrent>>,
}

struct SwapchainCurrent {
    frame: wgpu::SurfaceTexture,
    view: Arc<wgpu::TextureView>,
}

impl SwapchainTexture {
    pub fn new(
        surface: wgpu::Surface,
        surface_format: wgpu::TextureFormat,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            surface,
            surface_format,
            surface_config: surface_config.into(),
            current: RwLock::new(None),
        }
    }

    pub fn update_current_texture(&self) -> Result<(), wgpu::SurfaceError> {
        let mut current = self.current.write().unwrap();

        assert!(
            current.is_none(),
            "A frame hasn't been fully presented yet!"
        );

        // TODO: explicit error handling here (swapchain may need to be recreated)
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        *current = Some(SwapchainCurrent {
            frame,
            view: Arc::new(view),
        });

        Ok(())
    }

    pub fn present(&self) {
        self.current
            .write()
            .unwrap()
            .take()
            .expect("Trying to present an unstarted frame")
            .frame
            .present();
    }

    /// Reconfigures the surface using a specified size
    pub fn reconfigure(&self, device: &wgpu::Device, new_size: IVec2) {
        let mut sc = self.surface_config.write().unwrap();
        sc.width = new_size.x as _;
        sc.height = new_size.y as _;
        self.surface.configure(device, &sc);
    }
}

impl RenderTarget for SwapchainTexture {
    fn get_current_view(&self) -> Arc<wgpu::TextureView> {
        self.current.read().unwrap().as_ref().unwrap().view.clone()
    }

    fn get_size(&self) -> IVec2 {
        let config = self.surface_config.read().unwrap();
        IVec2::new(config.width as i32, config.height as i32)
    }

    fn get_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }
}

use std::sync::Arc;
use glam::UVec2;
use log::*;
use winit::window::Window;

use super::RenderContext;

pub struct MainWindow {
    pub winit_window: Arc<Window>,
    pub surface: wgpu::Surface,
    pub surface_format: wgpu::TextureFormat,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub current: Option<CurrentFrame>,
}

pub struct CurrentFrame {
    pub frame: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}

impl MainWindow {
    /// Initializes all wgpu state for given window
    pub fn new(
        context: &RenderContext,
        surface: wgpu::Surface,
        winit_window: Arc<Window>,
    ) -> Self {
        let surface_format = surface.get_preferred_format(&context.adapter).unwrap();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: winit_window.inner_size().width,
            height: winit_window.inner_size().height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&context.device, &surface_config);

        Self {
            winit_window,
            surface,
            surface_format,
            surface_config,
            current: None,
        }
    }

    /// Begins a frame by fetching next image from the surface
    /// 
    /// ## Panics
    /// Panics if a previous frame wasn't properly shut down using `finish_frame`,
    /// or if wgpu reports an out of memory error.
    pub fn begin_frame(&mut self, context: &RenderContext) {
        assert!(self.current.is_none(), "Previous frame wasn't finished");

        let frame = match self.surface.get_current_texture() {
            Err(wgpu::SurfaceError::Lost) |
            Err(wgpu::SurfaceError::Outdated) => {
                self.reconfigure(
                    &context.device,
                    UVec2::new(
                        self.winit_window.inner_size().width,
                        self.winit_window.inner_size().height,
                    ),
                );

                // Hopefully that doesn't turn into a stack overflowfest 🤞
                self.begin_frame(context);
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => {
                warn!("Surface timeout reported!");
                return; // Retry next frame
            }
            other => other.unwrap(),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.current = Some(CurrentFrame { frame, view });
    }

    /// Finishes the current frame by presenting it.
    /// 
    /// ## Panics
    /// Panics if no frame is currently being processed
    pub fn finish_frame(&mut self) {
        self.current
            .take()
            .expect("Trying to present an unstarted frame")
            .frame
            .present();
    }

    /// Reconfigures the surface to use a new size
    pub fn reconfigure(&mut self, device: &wgpu::Device, new_size: UVec2) {
        let mut sc = &mut self.surface_config;
        sc.width = new_size.x as _;
        sc.height = new_size.y as _;
        self.surface.configure(device, &sc);
    }
}

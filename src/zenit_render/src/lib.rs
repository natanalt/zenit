use anyhow::{anyhow, bail};
use base::{context::RenderContext, screen::Screen, swapchain::SwapchainTexture};
use log::*;
use pollster::FutureExt;
use std::sync::Arc;
use winit::window::Window;
use zenit_utils::{ok, AnyResult};

pub mod base;
pub mod example;
pub mod layers;

pub struct Renderer {
    pub context: Arc<RenderContext>,
    pub screens: Vec<Screen>,

    pub main_winit_window: Arc<Window>,
    pub main_window: Arc<SwapchainTexture>,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> AnyResult<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .ok_or(anyhow!("Couldn't find a graphics device"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .block_on()?;

        let surface_format = surface.get_preferred_format(&adapter).unwrap();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            context: Arc::new(RenderContext { device, queue }),
            screens: vec![],
            main_window: Arc::new(SwapchainTexture::new(
                surface,
                surface_format,
                surface_config,
            )),
            main_winit_window: window,
        })
    }

    pub fn render_frame(&self) -> AnyResult {
        if self.screens.is_empty() {
            bail!("Cannot render without any screens");
        }

        if let Err(err) = self.main_window.update_current_texture() {
            match err {
                wgpu::SurfaceError::Lost => {
                    let mut config = self.main_window.surface_config.write().unwrap();
                    config.width = self.main_winit_window.inner_size().width;
                    config.height = self.main_winit_window.inner_size().height;
                    self.main_window
                        .surface
                        .configure(&self.context.device, &config);
                }
                wgpu::SurfaceError::OutOfMemory => Err(err)?,

                // Ignore?
                wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Outdated => {
                    warn!("Main window swapchain timed out or is outdated");
                }
            }

            // Retry again next frame
            return ok();
        }

        let mut buffers = vec![];
        for screen in &self.screens {
            if screen.layers.is_empty() {
                bail!("Trying to render a layerless screen (`{:?}`)", screen.label);
            }

            for layer in &screen.layers {
                buffers.extend(layer.render( &self.context, screen.target.as_ref()));
            }
        }

        self.context.queue.submit(buffers);
        self.main_window.present();
        ok()
    }
}

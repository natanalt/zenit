//! Zenit Vulkan renderer
//! It's BAD.
//!

use crate::engine::Engine;
use log::info;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference,
    PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture,
    TextureUsages,
};
use winit::window::Window;

pub struct Renderer {
    pub adapter: Adapter,
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub sconfig: SurfaceConfiguration,
    pub surface_texture: Option<SurfaceTexture>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        info!("Initializing the renderer...");

        let wsize = window.inner_size();

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Couldn't find an adapter!");

        info!("Using adapter: {}", &adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let sconfig = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: wsize.width,
            height: wsize.height,
            present_mode: PresentMode::Fifo,
        };

        surface.configure(&device, &sconfig);

        Self {
            adapter,
            surface,
            device,
            queue,
            sconfig,
            surface_texture: None,
        }
    }

    pub fn begin_frame(&mut self, engine: &mut Engine) {
        if let Some(new_size) = engine.events.new_size {
            assert!(new_size.x != 0 && new_size.y != 0, "New size W=0 or H=0!");
            self.sconfig.width = new_size.x as _;
            self.sconfig.height = new_size.y as _;
            self.surface.configure(&self.device, &self.sconfig);
        }

        self.surface_texture = Some(
            self.surface
                .get_current_texture()
                .expect("Couldn't get next surface texture"),
        );
    }

    pub fn finish_frame(&mut self, _engine: &mut Engine) {
        if let Some(frame) = self.surface_texture.take() {
            frame.present();
        }
    }
}

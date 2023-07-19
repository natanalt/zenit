use log::*;
use pollster::FutureExt;
use std::sync::Arc;
use wgpu::{Features, Limits};
use winit::window::Window;

/// The device context contains public information regarding the current [`wgpu`] instance,
/// including the device, queue, adapter, main surface, etc.
///
/// ## Storage notes
/// The [`DeviceContext`] is stored within the [`Renderer`] and [`system::RenderSystem`] via a
/// shared [`Arc`] reference. No other places are allowed to have their own references to it,
/// strong or weak. The reason is that in the future the engine will support dynamic reconfiguration
/// of graphical settings, which will require the renderer to be capable of a full restart at any
/// point.
pub struct DeviceContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

impl DeviceContext {
    /// Creates a new [`wgpu`] instance and initializes a whole device context based from that.
    pub fn create(window: &Window) -> (Arc<Self>, wgpu::Surface, wgpu::SurfaceConfiguration) {
        info!("Creating a device context...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("couldn't create the surface")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("couldn't find a GPU");

        info!("Using adapter: {}", adapter.get_info().name);
        info!("Using backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    // BC compression, aka DXTn or S3
                    features: Features::TEXTURE_COMPRESSION_BC | Features::CLEAR_TEXTURE,
                    limits: Limits::default(),
                },
                None,
            )
            .block_on()
            .expect("couldn't initialize the device");

        // We could establish better error handling here
        device.on_uncaptured_error(Box::new(|error| {
            error!("An error has been reported by wgpu!");
            error!("{error}");
            panic!("Graphics API error: {error}");
        }));

        let sconfig = surface
            .get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            )
            .expect("surface unsupported by adapter");

        surface.configure(&device, &sconfig);

        let result = Arc::new(Self {
            device,
            queue,
            instance,
            adapter,
        });

        (result, surface, sconfig)
    }
}

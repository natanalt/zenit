use super::{DeviceContext, Renderer, frame_state::FrameState};
use crate::engine::{EngineContext, GlobalContext, System};
use log::*;
use parking_lot::Mutex;
use pollster::FutureExt;
use std::{iter, sync::Arc};
use wgpu::*;
use winit::window::Window;

pub struct RenderSystem {
    /// Note, this Arc should never be cloned - only 2/0 strong/weak references are allowed.
    dc: Arc<DeviceContext>,
    
    pub window: Arc<Window>,
    pub surface: Surface,
    pub sconfig: SurfaceConfiguration,

    pub pending_frame: Option<FrameState>,

    //pub skybox_renderer: SkyboxRenderer,
}

impl RenderSystem {
    pub fn new(window: Arc<Window>) -> Self {
        info!("Starting up the renderer...");

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("couldn't create the surface")
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("couldn't find a GPU");

        info!("Using adapter: {}", adapter.get_info().name);
        info!("Using backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    // BC compression, aka DXTn or S3
                    features: Features::TEXTURE_COMPRESSION_BC | Features::CLEAR_TEXTURE,
                    limits: Limits::default(),
                },
                None,
            )
            .block_on()
            .expect("couldn't initialize the device");

        let sconfig = surface
            .get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            )
            .expect("surface unsupported by adapter");

        surface.configure(&device, &sconfig);

        let dc = Arc::new(DeviceContext {
            device,
            queue,
            instance,
            adapter,
        });

        trace!("Loading basic resources...");
        //let skybox_renderer = SkyboxRenderer::new(&dc);

        Self {
            window,
            dc,
            surface,
            sconfig,
            
            pending_frame: None,

            //skybox_renderer,
        }
    }
}

impl System for RenderSystem {
    fn label(&self) -> &'static str {
        "Render System"
    }

    fn init(&mut self, ec: &mut EngineContext) {
        let gc = ec.global_context.get_mut();
        gc.renderer = Some(Arc::new(Mutex::new(Renderer::new(self.dc.clone()))));
    }

    fn main_process(&mut self, _ec: &EngineContext, _gc: &GlobalContext) {
        let Some(pending_frame) = self.pending_frame.take() else { return };

        let device = &self.dc.device;
        let queue = &self.dc.queue;

        // TODO: check for window size changes

        // Notes on swapchain fetches:
        //  - On some GPUs on Linux under Mesa, Vulkan swapchains may randomly timeout. As
        //    a countermeasure, timeouts on Linux do nothing, and just skip the frame, hoping that
        //    the swapchain will go back to working next frame.
        //    In the future, Zenit could check for N timeouts in a row (maybe N=150 in a row),
        //    and then panic. For now it'll keep on ignoring timeouts.
        //
        //    Outside Linux a timeout just causes a panic.
        //    Reference: https://github.com/gfx-rs/wgpu/issues/1218
        //  - SurfaceError::Lost currently panics.
        //    Veloren's renderer handles it differently: in cases of ::Outdated or ::Lost, it
        //    just recreates the whole swapchain. We may do the same someday.
        let current_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,

            Err(SurfaceError::Outdated) => {
                self.sconfig.width = self.window.inner_size().width;
                self.sconfig.height = self.window.inner_size().height;
                self.surface.configure(device, &self.sconfig);

                // Try again next frame
                return;
            }

            // Perhaps SurfaceError::Lost may be recoverable if the surface is recreated
            Err(SurfaceError::Lost) => panic!("swapchain failure: surface lost"),
            Err(SurfaceError::OutOfMemory) => panic!("swapchain failure: out of memory"),

            #[cfg(not(target_os = "linux"))]
            Err(SurfaceError::Timeout) => panic!("swapchain failure: timeout"),
            #[cfg(target_os = "linux")]
            Err(SurfaceError::Timeout) => {
                // Try again next frame
                return;
            }
        };

        let current_view = current_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Framebuffer view"),
            ..Default::default()
        });

        let mut encoder = device.create_command_encoder(&Default::default());

        for (camera, scene) in pending_frame.targets {
            let resources = camera.gpu_resources.lock();
            
            // TODO: actually render something
            encoder.clear_texture(&resources.texture, &ImageSubresourceRange {
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
        }

        if let Some(target) = pending_frame.screen_target {
            let info = target.gpu_resources.lock();
            encoder.copy_texture_to_texture(
                info.texture.as_image_copy(),
                current_texture.texture.as_image_copy(),
                Extent3d {
                    width: info.texture.width().min(current_texture.texture.width()),
                    height: info.texture.height().min(current_texture.texture.height()),
                    depth_or_array_layers: 1,
                },
            )
        } else {
            // Do *something* just in case this happens to be the only operation this frame
            // (It's rather unlikely but whatever)
            encoder.clear_texture(
                &current_texture.texture,
                &ImageSubresourceRange {
                    aspect: TextureAspect::All,
                    ..Default::default()
                },
            );
        }

        queue.submit(iter::once(encoder.finish()));
        current_texture.present();
    }

    fn post_process(&mut self, ec: &EngineContext) {
        let gc = ec.global_context.read();
        let uni = gc.read_universe();
        let renderer = gc.lock_renderer();

        self.pending_frame = Some(FrameState::from_ecs(&uni, &renderer));
    }
}

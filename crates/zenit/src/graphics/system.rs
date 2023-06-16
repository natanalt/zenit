use super::{egui_support, frame_state::FrameState, DeviceContext, Renderer};
use crate::{
    engine::{EngineContext, GlobalState, System},
    entities::Universe,
    graphics::SkyboxRenderer,
};
use egui::{ClippedPrimitive, FullOutput};
use log::*;
use parking_lot::Mutex;
use pollster::FutureExt;
use std::{mem, sync::Arc};
use wgpu::*;
use winit::window::Window;

pub struct RenderSystem {
    /// Note, this Arc should never be cloned - only 2/0 strong/weak references are allowed.
    dc: Arc<DeviceContext>,

    pub window: Arc<Window>,
    pub surface: Surface,
    pub sconfig: SurfaceConfiguration,

    pub last_egui_output: EguiOutput,
    pub egui_output: Arc<Mutex<EguiOutput>>,
    pub egui_renderer: egui_support::EguiSupport,

    pub pending_frame: Option<FrameState>,
    pub skybox_renderer: SkyboxRenderer,
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

        let dc = Arc::new(DeviceContext {
            device,
            queue,
            instance,
            adapter,
        });

        trace!("Loading basic resources...");
        let skybox_renderer = SkyboxRenderer::new(&dc);

        Self {
            egui_renderer: egui_support::EguiSupport::new(&dc, &sconfig),

            window,
            dc,
            surface,
            sconfig,
            last_egui_output: Default::default(),
            egui_output: Default::default(),
            pending_frame: None,
            skybox_renderer,
        }
    }
}

impl System for RenderSystem {
    fn label(&self) -> &'static str {
        "Render System"
    }

    fn init(&mut self, ec: &mut EngineContext) {
        trace!("Initializing the global renderer...");
        let gc = ec.globals.get_mut();
        gc.add_lockable(Renderer::new(self.dc.clone(), self.egui_output.clone()));
    }

    fn main_process(&mut self, _ec: &EngineContext, gc: &GlobalState) {
        let Some(pending_frame) = self.pending_frame.take() else { return };

        // Debug check: verify that only the renderer and render system have a DC reference.
        if cfg!(debug_assertions) {
            let expected_refs = match gc.exists::<Mutex<Renderer>>() {
                true => 2,
                false => 1,
            };
            assert!(
                Arc::strong_count(&self.dc) == expected_refs && Arc::weak_count(&self.dc) == 0,
                "only the renderer and render system are allowed to have live DC references"
            );
        }

        let device = &self.dc.device;
        let queue = &self.dc.queue;

        // TODO: check for window size changes

        // Notes on swapchain fetches:
        //  - On some GPUs on Linux under Mesa, Vulkan swapchains may randomly timeout. As
        //    a countermeasure, timeouts have a Linux-specific implementation that ignores
        //    these errors, in hope that the issue is resolved next frame.
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

            // TODO: Perhaps SurfaceError::Lost may be recoverable if the surface is recreated
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

        for (camera, scene) in pending_frame.targets {
            let camera_resources = camera.gpu_resources.lock();

            if let Some(skybox_resources) = &scene.skybox_resources {
                let skybox_resources = skybox_resources.lock();
                self.skybox_renderer.render_skybox(
                    &self.dc,
                    &skybox_resources,
                    &camera_resources.texture_view,
                );
            }
        }

        let textures_delta = mem::take(&mut self.last_egui_output.full_output.textures_delta);
        let shapes = mem::take(&mut self.last_egui_output.tesselated);

        queue.submit(self.egui_renderer.render(
            &self.dc,
            &self.window,
            &current_view,
            textures_delta,
            shapes,
        ));
        current_texture.present();
    }

    fn post_process(&mut self, ec: &EngineContext) {
        let gc = ec.globals.read();
        let uni = gc.read::<Universe>();
        let mut renderer = gc.lock::<Renderer>();

        self.pending_frame = Some(FrameState::from_ecs(&uni, &renderer));
        self.last_egui_output = mem::take(&mut self.egui_output.lock());
        renderer.collect_garbage();
    }
}

#[derive(Default)]
pub struct EguiOutput {
    pub full_output: FullOutput,
    pub tesselated: Vec<ClippedPrimitive>,
}

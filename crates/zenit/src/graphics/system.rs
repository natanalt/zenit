use super::{
    imgui_renderer::ImguiRenderer, BuiltScene, CameraGpuResources, DeviceContext, PendingFrame,
    Renderer,
};
use crate::{
    engine::{EngineContext, GlobalState, System},
    graphics::SkyboxRenderer,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use std::{mem, sync::Arc};
use wgpu::*;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

/// The render system handles submitting render commands and displaying everything in general.
pub struct GraphicsSystem {
    /// Note, this Arc should never be cloned - only 2/0 strong/weak references are allowed.
    dc: Arc<DeviceContext>,

    window: Arc<Window>,
    surface: Surface,
    sconfig: SurfaceConfiguration,

    imgui_renderer: ImguiRenderer,
    skybox_renderer: SkyboxRenderer,

    pending_frame: Option<PendingFrame>,
}

impl GraphicsSystem {
    pub fn new(
        renderer: &mut Renderer,
        dc: Arc<DeviceContext>,
        window: Arc<Window>,
        surface: Surface,
        sconfig: SurfaceConfiguration,
    ) -> Self {
        CameraGpuResources::register_camera_bind_layout(renderer);

        let imgui_renderer = ImguiRenderer::new(renderer, sconfig.format);
        let skybox_renderer = SkyboxRenderer::new(renderer);

        Self {
            dc,
            window,
            surface,
            sconfig,
            imgui_renderer,
            skybox_renderer,
            pending_frame: None,
        }
    }
}

impl System for GraphicsSystem {
    fn label(&self) -> &'static str {
        "Graphics System"
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

        for message in gc.new_messages_of::<WindowEvent>() {
            if let WindowEvent::Resized(_new_size) = message {
                self.reconfigure_surface();
                return;
            }
        }

        {
            let PhysicalSize { width, height } = self.window.inner_size();
            if width == 0 || height == 0 {
                return;
            }

            if width != self.sconfig.width || height != self.sconfig.height {
                self.reconfigure_surface();
                return;
            }
        }

        let queue = &self.dc.queue;

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
                self.reconfigure_surface();

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
                // Try again next frame, see comment above
                return;
            }
        };

        let current_view = current_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Framebuffer view"),
            ..Default::default()
        });

        let mut command_buffers = Vec::with_capacity(pending_frame.scenes.len() + 1);

        // Render all pending scenes in parallel
        pending_frame
            .scenes
            .into_par_iter()
            .map(|scene| self.render_scene(scene))
            .collect_into_vec(&mut command_buffers);

        // TODO: add this to the above parallel generator
        if let Some(imgui_data) = pending_frame.imgui_render_data {
            command_buffers.push(self.imgui_renderer.render_imgui(
                &self.dc,
                imgui_data,
                &current_view,
            ));
        }

        queue.submit(command_buffers.into_iter());
        current_texture.present();
    }

    fn post_process(&mut self, ec: &EngineContext) {
        let gc = ec.globals.read();
        let mut renderer = gc.lock::<Renderer>();

        let mut pending_frame = mem::take(&mut renderer.pending_frame);
        self.imgui_renderer.setup_textures(
            &mut renderer,
            pending_frame.imgui_new_textures.drain(..),
            pending_frame.imgui_new_font.take(),
        );

        self.pending_frame = Some(pending_frame);

        renderer.collect_garbage();
    }
}

impl GraphicsSystem {
    /// Called when the window surface changed, for example by getting resized.
    ///
    /// In the future, this function will also handle reloading of resources dependent on the surface
    /// dimensions.
    fn reconfigure_surface(&mut self) {
        self.sconfig.width = self.window.inner_size().width;
        self.sconfig.height = self.window.inner_size().height;
        self.surface.configure(&self.dc.device, &self.sconfig);
    }

    /// Encodes a scene into a command buffer.
    fn render_scene(&self, scene: BuiltScene) -> wgpu::CommandBuffer {
        let device = &self.dc.device;

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("scene encoder"),
        });

        encoder.push_debug_group("scene rendering");

        for (camera, _transform) in scene.targets {
            encoder.push_debug_group("camera instance");

            let camera_resources = camera.lock();

            if let Some(skybox_resources) = &scene.skybox {
                let skybox_resources = skybox_resources.lock();
                self.skybox_renderer.render_skybox(
                    &self.dc,
                    &mut encoder,
                    &camera_resources,
                    &skybox_resources,
                );
            }

            encoder.pop_debug_group();
        }

        encoder.pop_debug_group();
        encoder.finish()
    }
}

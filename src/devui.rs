//! DevUI, aka. the egui layer
//!
//! It is currently dependent on there being a wgpu renderer, which is the only one at the moment.
//! Parts of this code will need to be moved if another renderer is ever created.
//!

use egui::CtxRef;
use wgpu::{CommandBuffer, CommandEncoderDescriptor, TextureViewDescriptor};
use winit::{event::WindowEvent, window::Window};

use crate::{engine::Engine, renderer::Renderer};

pub struct DevUI {
    /// If false, the UI won't be processed or rendered
    pub show: bool,
    pub context: CtxRef,
    pub egui_rpass: egui_wgpu_backend::RenderPass,
    pub winit_state: egui_winit::State,
}

impl DevUI {
    pub fn new(renderer: &Renderer, window: &Window) -> Self {
        Self {
            show: true,
            context: CtxRef::default(),
            egui_rpass: egui_wgpu_backend::RenderPass::new(
                &renderer.device,
                renderer
                    .surface
                    .get_preferred_format(&renderer.adapter)
                    .unwrap(),
                1,
            ),
            winit_state: egui_winit::State::new(window),
        }
    }

    pub fn process_event(&mut self, event: &WindowEvent) -> bool {
        if self.show {
            self.winit_state.on_event(&self.context, event)
        } else {
            false
        }
    }

    pub fn frame(&mut self, engine: &mut Engine, window: &Window) -> Option<CommandBuffer> {
        if !self.show {
            return None;
        }

        let context = &mut self.context;
        let input = self.winit_state.take_egui_input(&window);
        let (output, clips) = context.run(input, |ctx| {
            // something will happen here someday
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("hello there");
            });
        });
        self.winit_state.handle_output(&window, context, output);
        let meshes = context.tessellate(clips);

        // Potential TODO: render to a second texture to allow blending between the game and devui?
        let renderer = engine.renderer.as_ref().expect("No renderer?");
        let texture = &renderer
            .surface_texture
            .as_ref()
            .expect("No target texture?")
            .texture
            .create_view(&TextureViewDescriptor::default());

        let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
            physical_width: renderer.sconfig.width,
            physical_height: renderer.sconfig.height,
            scale_factor: window.scale_factor() as f32,
        };

        self.egui_rpass
            .update_texture(&renderer.device, &renderer.queue, &context.font_image());
        self.egui_rpass
            .update_user_textures(&renderer.device, &renderer.queue);
        self.egui_rpass.update_buffers(
            &renderer.device,
            &renderer.queue,
            &meshes,
            &screen_descriptor,
        );

        let mut encoder = renderer
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        self.egui_rpass
            .execute(
                &mut encoder,
                &texture,
                &meshes,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .expect("egui rendering failed");

        Some(encoder.finish())
    }
}

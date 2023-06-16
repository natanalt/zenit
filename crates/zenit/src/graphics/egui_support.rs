use super::DeviceContext;
use egui_wgpu::renderer::ScreenDescriptor;
use wgpu::{
    Color, CommandEncoderDescriptor, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    SurfaceConfiguration, TextureView,
};
use winit::window::Window;

/// Wrapper around the incredibly annoying API of [`egui_wgpu`].
pub struct EguiSupport {
    renderer: egui_wgpu::Renderer,
}

impl EguiSupport {
    pub fn new(dc: &DeviceContext, sconfig: &SurfaceConfiguration) -> Self {
        Self {
            renderer: egui_wgpu::Renderer::new(
                &dc.device,
                sconfig.format,
                None,
                1, // TODO: msaa
            ),
        }
    }

    pub fn render(
        &mut self,
        dc: &DeviceContext,
        window: &Window,
        target: &TextureView,
        textures_delta: egui::TexturesDelta,
        shapes: Vec<egui::ClippedPrimitive>,
    ) -> Vec<wgpu::CommandBuffer> {
        let descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, delta) in textures_delta.set {
            self.renderer
                .update_texture(&dc.device, &dc.queue, id, &delta);
        }

        let mut encoder = dc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("egui encoder"),
        });

        let mut buffers =
            self.renderer
                .update_buffers(&dc.device, &dc.queue, &mut encoder, &shapes, &descriptor);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target,
                resolve_target: None, // TODO: MSAA
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        self.renderer.render(&mut pass, &shapes, &descriptor);
        drop(pass);

        for id in textures_delta.free {
            self.renderer.free_texture(&id);
        }

        buffers.push(encoder.finish());
        buffers
    }
}

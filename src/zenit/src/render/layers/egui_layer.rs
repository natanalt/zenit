use crate::{
    ctpanel::EguiManager,
    render::base::{context::RenderContext, screen::RenderLayer, target::RenderTarget},
};
use egui::mutex::Mutex;
use egui_wgpu::renderer::RenderPass as EguiRenderPass;
use std::sync::Arc;

/// A layer for integrating egui
pub struct EguiLayer {
    pub egui_manager: Arc<EguiManager>,
    pub rpass: Mutex<EguiRenderPass>,
}

impl EguiLayer {
    pub fn new(
        device: &wgpu::Device,
        context: Arc<EguiManager>,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            egui_manager: context,
            rpass: Mutex::new(EguiRenderPass::new(device, output_format, 1)),
        }
    }
}

impl RenderLayer for EguiLayer {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Egui Command Encoder"),
            });

        let mut rpass = self.rpass.lock();
        let last_output = self.egui_manager.last_output.read().unwrap();

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [target.get_size().x as u32, target.get_size().y as u32],
            pixels_per_point: *self.egui_manager.pixels_per_point.read().unwrap(),
        };

        for (id, image_delta) in &last_output.textures_delta.set {
            rpass.update_texture(&context.device, &context.queue, *id, image_delta);
        }

        let tesselated = self
            .egui_manager
            .context
            .tessellate(last_output.shapes.clone());

        rpass.update_buffers(
            &context.device,
            &context.queue,
            &tesselated,
            &screen_descriptor,
        );

        rpass.execute(
            &mut encoder,
            target.get_current_view().as_ref(),
            &tesselated,
            &screen_descriptor,
            Some(wgpu::Color::BLACK),
        );

        for id in &last_output.textures_delta.free {
            rpass.free_texture(id);
        }

        vec![encoder.finish()]
    }
}

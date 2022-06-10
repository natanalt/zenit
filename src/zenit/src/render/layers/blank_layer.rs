use crate::render::base::{
    context::RenderContext, screen::RenderLayer, target::RenderTarget, utils,
};
use std::sync::Arc;
use zenit_utils::AnyResult;

pub struct BlankLayer {
    pub clear_color: wgpu::Color,
    pub pipeline: wgpu::RenderPipeline,
}

impl BlankLayer {
    pub fn new(context: &Arc<RenderContext>, format: wgpu::TextureFormat) -> AnyResult<Self> {
        let shader = crate::include_shader!(context.device, "blank.shader");
        Ok(Self {
            clear_color: wgpu::Color::BLACK,
            pipeline: context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Blank pipeline"),
                    layout: None,
                    vertex: wgpu::VertexState {
                        module: &shader.module,
                        entry_point: "main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader.module,
                        entry_point: "main",
                        targets: &[format.into()],
                    }),
                    primitive: utils::USUAL_PRIMITIVES,
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                }),
        })
    }
}

impl RenderLayer for BlankLayer {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let view = target.get_current_view();
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        drop(rpass);

        vec![encoder.finish()]
    }
}

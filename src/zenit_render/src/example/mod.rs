use crate::base::{context::RenderContext, screen::RenderLayer, target::RenderTarget, utils};
use byteorder::{NativeEndian, WriteBytesExt};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use zenit_utils::AnyResult;

pub struct TriangleLayer {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
}

impl TriangleLayer {
    pub fn new(context: &Arc<RenderContext>, format: wgpu::TextureFormat) -> AnyResult<Self> {
        const BUFFER: &[f32] = &[
            0.0, 0.5, 1.0, 0.0, 0.0, 1.0, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, -0.5, -0.5, 0.0, 0.0, 1.0,
            1.0,
        ];

        let raw_buffer = {
            let mut buffer = vec![];
            let mut cursor = std::io::Cursor::new(&mut buffer);
            for &value in BUFFER {
                cursor.write_f32::<NativeEndian>(value).unwrap();
            }
            buffer
        };

        let shader = crate::include_shader!(context.device, "example_triangle.shader");

        let layout = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Triangle pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: "main",
                    buffers: &[crate::single_vertex_buffer![
                        0 => Float32x2,
                        1 => Float32x4,
                    ]],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: utils::USUAL_PRIMITIVES,
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let vertices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle vertex buffer"),
                contents: &raw_buffer,
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(Self { pipeline, vertices })
    }
}

impl RenderLayer for TriangleLayer {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let view = target.get_current_view();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.15,
                        g: 0.0,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..3, 0..1);
        drop(pass);

        vec![encoder.finish()]
    }
}

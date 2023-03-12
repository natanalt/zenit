use crate::render::DeviceContext;
use glam::*;
use std::sync::Arc;
use wgpu::*;

use super::TextureCubemap;

pub enum Skybox {
    Textured {
        texture: Arc<TextureCubemap>,
        bind_group: BindGroup,
    },
    FlatColored {
        color: Vec4,
    },
}

pub struct SkyboxRenderer {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub vertex_buffer: Buffer,
}

impl SkyboxRenderer {
    pub fn new(dc: &DeviceContext) -> Self {
        let bind_group_layout = Arc::new(dc.device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("Skybox Bind Group Layout"),
                entries: &crate::bind_group_layout_array![
                    0 => (
                        FRAGMENT,
                        BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: TextureViewDimension::Cube,
                            multisampled: false,
                        }
                    ),
                    1 => (
                        FRAGMENT,
                        BindingType::Sampler(SamplerBindingType::Filtering),
                    ),
                ],
            },
        ));

        Self {
            pipeline: todo!(),
            bind_group_layout,
            vertex_buffer: todo!(),
        }
    }

    pub fn render(
        &self,
        skybox: &Skybox,
        target: &TextureView,
        common_group: &BindGroup,
        encoder: &mut CommandEncoder,
    ) {
        let clear_color = match skybox {
            Skybox::Textured { .. } => vec4(0.0, 0.0, 0.0, 1.0),
            Skybox::FlatColored { color } => *color,
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Skybox Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: clear_color.x as _,
                        g: clear_color.y as _,
                        b: clear_color.z as _,
                        a: clear_color.w as _,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        // If the skybox is flat colored, the pass will still happen, but no vertices will actually
        // be passed for rendering. This will effectively work as a flat fill of the target.
        if let Skybox::Textured {
            texture: _,
            bind_group,
        } = skybox
        {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_bind_group(0, common_group, &[]);
            pass.set_bind_group(1, bind_group, &[]);
            pass.draw(0..SKYBOX_UV_COUNT as u32, 0..1);
        }
    }
}

// TODO: move skybox UVs to shader?
/// Contents of the skybox buffer created at startup.
/// 
/// Borrowed from https://learnopengl.com/Advanced-OpenGL/Cubemaps
/// (section *"Displaying a skybox"*)
#[rustfmt::skip]
const SKYBOX_UVS: &[f32] = &[
    -1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    
    -1.0, -1.0,  1.0,
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    -1.0, -1.0,  1.0,
    
     1.0, -1.0, -1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0, -1.0,
     1.0, -1.0, -1.0,
    
    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
     1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,
    
    -1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,
     1.0,  1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,
    
    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0,
];
const SKYBOX_UV_COUNT: usize = SKYBOX_UVS.len() / 3;

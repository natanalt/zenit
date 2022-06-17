use super::camera::CameraUniform;
use crate::{
    include_shader,
    render::base::{utils::USUAL_PRIMITIVES, RenderContext},
    single_vertex_buffer,
};
use crevice::std140::AsStd140;
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

/// Skybox pipeline implements a simple skybox
pub struct SkyboxPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertices: wgpu::Buffer,
}

impl SkyboxPipeline {
    pub fn new(context: &RenderContext, format: wgpu::TextureFormat) -> Self {
        let shader = include_shader!(context.device, "skybox.shader");

        let bindings = shader
            .metadata
            .get("bindings")
            .map(|v| v.as_table())
            .flatten()
            .expect("Invalid skybox shader metadata");
        let camera_binding = bindings["camera"].as_integer().unwrap();
        let texture_binding = bindings["skybox_texture"].as_integer().unwrap();

        let group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Skybox pipeline bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: camera_binding as _,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: NonZeroU64::new(
                                    CameraUniform::std140_size_static() as u64,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: texture_binding as _,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Skybox pipeline layout"),
                    bind_group_layouts: &[&group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Skybox pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: "main",
                    buffers: &[single_vertex_buffer![
                        0 => Float32x3,
                    ]],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    }],
                }),
                primitive: USUAL_PRIMITIVES,
                depth_stencil: None,
                multiview: None,
                multisample: wgpu::MultisampleState::default(),
            });

        let vertices = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Skybox vertices"),
                contents: &zenit_utils::pack_floats(SKYBOX_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Self { pipeline, vertices }
    }
}

#[rustfmt::skip]
const SKYBOX_VERTICES: &[f32] = &[
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
     1.0, -1.0,  1.0
];

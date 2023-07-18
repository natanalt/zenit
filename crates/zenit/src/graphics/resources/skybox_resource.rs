use crate::bind_group_layout_array;
use crate::graphics::{DeviceContext, Renderer};
use byteorder::{NativeEndian, WriteBytesExt};
use glam::*;
use parking_lot::Mutex;
use std::mem;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::*;
use zenit_utils::ArcPoolHandle;

use super::{CameraGpuResources, CubemapHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkyboxHandle(pub(in crate::graphics) ArcPoolHandle);

pub struct SkyboxResource {
    pub label: String,
    pub(in crate::graphics) gpu_resources: Arc<Mutex<SkyboxGpuResources>>,
}

impl SkyboxResource {
    pub fn new(r: &mut Renderer, desc: &SkyboxDescriptor) -> Self {
        use SkyboxBackground::*;

        // That's a lot of indents but ehhhhhhh
        Self {
            label: desc.name.clone(),
            gpu_resources: Arc::new(Mutex::new(SkyboxGpuResources {
                background: desc.background.clone(),
                bind_group: match &desc.background {
                    Textured(handle) => {
                        let cubemap = r.cubemaps.get(handle);

                        Some(
                            r.dc.device.create_bind_group(&BindGroupDescriptor {
                                label: Some("skybox bind group"),
                                layout: r
                                    .shared_bind_layouts
                                    .get("Skybox")
                                    .expect("skybox bind layout missing"),
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(&cubemap.view),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(
                                            match cubemap.unfiltered {
                                                true => &r.unfiltered_sampler,
                                                false => &r.filtered_sampler,
                                            },
                                        ),
                                    },
                                ],
                            }),
                        )
                    }
                    Solid(_) => None,
                },
            })),
        }
    }
}

pub struct SkyboxGpuResources {
    pub background: SkyboxBackground,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct SkyboxRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

impl SkyboxRenderer {
    pub fn new(r: &mut Renderer) -> Self {
        let shader = r.shaders.get("skybox").expect("skybox shader missing");

        let camera_bind_layout = r
            .shared_bind_layouts
            .get("Camera")
            .expect("missing camera bind layout");

        let skybox_bind_layout =
            r.dc.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Skybox Bind Group Layout"),
                    entries: &bind_group_layout_array![
                        0 => (
                            FRAGMENT,
                            BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::Cube,
                                multisampled: false,
                            },
                        ),
                        1 => (
                            FRAGMENT,
                            BindingType::Sampler(SamplerBindingType::Filtering),
                        ),
                    ],
                });

        let pipeline =
            r.dc.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("skybox pipeline"),
                    layout: Some(
                        &r.dc
                            .device
                            .create_pipeline_layout(&PipelineLayoutDescriptor {
                                label: Some("skybox pipeline layout"),
                                bind_group_layouts: &[camera_bind_layout, &skybox_bind_layout],
                                push_constant_ranges: &[],
                            }),
                    ),
                    vertex: VertexState {
                        module: &shader.module,
                        entry_point: "vs_main",
                        buffers: &[VertexBufferLayout {
                            array_stride: mem::size_of::<Vec3>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &vertex_attr_array![0 => Float32x3],
                        }],
                    },
                    fragment: Some(FragmentState {
                        module: &shader.module,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: crate::graphics::RENDER_FORMAT,
                            blend: None,
                            write_mask: ColorWrites::all(),
                        })],
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Cw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        // The skybox does not really care about the depth buffer
                        // It just clears it.
                        format: crate::graphics::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Always,
                        stencil: StencilState {
                            front: StencilFaceState::IGNORE,
                            back: StencilFaceState::IGNORE,
                            read_mask: 0xff,
                            write_mask: 0xff,
                        },
                        bias: DepthBiasState {
                            constant: 0,
                            slope_scale: 0.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: MultisampleState::default(),
                    multiview: None,
                });

        let vertex_data = {
            // A bit hacky way to transfer those constants into a u8 byte array without
            // weird unsafe fuckery
            let mut data = Vec::with_capacity(SKYBOX_VERTICES.len() * 4 * 3);
            let mut cursor = std::io::Cursor::new(&mut data);

            for &vertex in SKYBOX_VERTICES {
                cursor.write_f32::<NativeEndian>(vertex).unwrap();
            }

            data
        };

        let vertex_buffer = r.dc.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("skybox vertex buffer"),
            contents: &vertex_data,
            usage: BufferUsages::VERTEX,
        });

        r.shared_bind_layouts
            .insert("Skybox".to_string(), skybox_bind_layout);

        Self {
            pipeline,
            vertex_buffer,
        }
    }

    pub fn render_skybox(
        &self,
        _dc: &DeviceContext,
        encoder: &mut CommandEncoder,
        camera: &CameraGpuResources,
        skybox: &SkyboxGpuResources,
    ) {
        use SkyboxBackground::*;
        let clear_color = match &skybox.background {
            Textured(_) => vec4(0.0, 0.0, 0.0, 1.0),
            Solid(color) => *color,
        };

        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("skybox render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &camera.texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: clear_color.x as f64,
                        g: clear_color.y as f64,
                        b: clear_color.z as f64,
                        a: clear_color.w as f64,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &camera.depth_texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        match &skybox.background {
            Textured(_) => {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(0, &camera.bind_group, &[]);
                rpass.set_bind_group(1, skybox.bind_group.as_ref().unwrap(), &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.draw(0..36, 0..1);
            }
            Solid(_) => {
                // We only need to clear
            }
        }
    }
}

pub struct SkyboxDescriptor {
    pub name: String,
    pub background: SkyboxBackground,
}

#[derive(Clone)]
pub enum SkyboxBackground {
    Textured(CubemapHandle),
    Solid(Vec4),
}

/// Contents of the skybox vertex buffer.
/// Borrowed from learnopengl.com.
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
     1.0, -1.0,  1.0,
];

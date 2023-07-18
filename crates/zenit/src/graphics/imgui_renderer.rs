//! Custom renderer for Dear ImGui
//!
//! It started as a fork of the following library, but it got modified so heavily, that at this point
//! I can really only call it inspired by it, as what we have is a total rewrite:
//!
//! <https://github.com/Yatekii/imgui-wgpu-rs>
//!

use super::{CameraHandle, DeviceContext, Renderer, TextureHandle};
use crate::bind_group_layout_array;
use ahash::AHashMap;
use glam::{vec2, Vec2};
use imgui::sys::{ImDrawIdx, ImDrawVert};
use imgui::{DrawCmdParams, DrawData, DrawVert, TextureId};
use std::num::NonZeroU64;
use std::ops::Range;
use std::{mem, slice};
use wgpu::*;
use zenit_utils::align;

/// Initial size for vertex/index buffers.
/// Set to 25 000 vertices cause why not, 500 KB per buffer really isn't too much.
const DEFAULT_BUFFER_SIZE: u64 = 20 * 25000;

pub struct ImguiTexture {
    /// The ID is used for tracking whether the underlying wgpu texture resource gets reallocated.
    /// This can happen for example when a camera changes its dimensions.
    ///
    /// If set to [`None`], it'll be updated with the current texture's ID on the next post process
    /// in the render system.
    id: Option<wgpu::Id<Texture>>,
    bind_group: Option<wgpu::BindGroup>,
    handle: ImTextureHandle,
}

// TODO: Remove this unsafe hack once wgpu gets updated
//       The reason I'm doing this is because wgpu::Id doesn't mark itself as Send+Sync, despite
//       only containing a NonZeroU64 (and a PhantomData<*mut Texture>, which is the core issue here
//       as !Send + !Sync)
//
//       https://github.com/gfx-rs/wgpu/pull/3801 fixes it and was merged, but at the time of
//       writing 0.16.1 is the latest version of wgpu, and it doesn't include that fix.
unsafe impl Send for ImguiTexture {}
unsafe impl Sync for ImguiTexture {}

impl ImguiTexture {
    pub fn from_texture(handle: TextureHandle) -> Self {
        Self {
            id: None,
            bind_group: None,
            handle: ImTextureHandle::Texture(handle),
        }
    }

    pub fn from_camera(handle: CameraHandle) -> Self {
        Self {
            id: None,
            bind_group: None,
            handle: ImTextureHandle::Camera(handle),
        }
    }

    /// Attempts to update the ImTexture's underlying bind group if:
    ///  * there's no existing bind group for this ImTexture,
    ///  * the underlying Zenit texture resource was reallocated.
    pub fn try_update_bind_group(&mut self, renderer: &mut Renderer, layout: &BindGroupLayout) {
        match &self.handle {
            ImTextureHandle::Texture(handle) => {
                let texture = renderer.textures.get(handle);
                let texture_id = texture.handle.global_id();

                if Some(texture_id) != self.id || self.bind_group.is_none() {
                    self.id = Some(texture_id);
                    self.bind_group =
                        Some(renderer.dc.device.create_bind_group(&BindGroupDescriptor {
                            label: Some("imgui texture bind group layout"),
                            layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(&texture.view),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(if texture.unfiltered {
                                        &renderer.unfiltered_sampler
                                    } else {
                                        &renderer.filtered_sampler
                                    }),
                                },
                            ],
                        }))
                }
            }
            ImTextureHandle::Camera(handle) => {
                let camera = renderer.cameras.get(handle);
                let gpu_resources = camera.gpu_resources.lock();

                let texture = &gpu_resources.texture;
                let texture_view = &gpu_resources.texture_view;
                let texture_id = texture.global_id();

                if Some(texture_id) != self.id || self.bind_group.is_none() {
                    self.id = Some(texture_id);
                    self.bind_group =
                        Some(renderer.dc.device.create_bind_group(&BindGroupDescriptor {
                            label: Some("imgui texture bind group layout"),
                            layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(texture_view),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(
                                        &renderer.unfiltered_sampler,
                                    ),
                                },
                            ],
                        }))
                }
            }
        }
    }
}

enum ImTextureHandle {
    Texture(TextureHandle),
    Camera(CameraHandle),
}

/// Structure sent to the ImGui renderer from the scene system. It contains all info necessary
/// for the renderer to process the frame, including the draw vertices, any updated fonts, any
/// new textures
pub struct ImguiFrame {
    /// List of new textures. If the [`TextureId`] is already present in the renderer, it will be
    /// overwritten with the new value.
    pub new_textures: Vec<(TextureId, ImguiTexture)>,
    /// Every texture on this list will be removed from the renderer.
    /// If somehow you add and remove a texture at once, the removal will be prioritized, as it happens
    /// after every new texture is added.
    pub textures_to_remove: Vec<TextureId>,
    /// Draw data as generated by Dear ImGui.
    pub draw_data: ImguiRenderData,
}

pub struct ImguiRenderer {
    textures: AHashMap<TextureId, ImguiTexture>,

    pipeline: wgpu::RenderPipeline,
    texture_bind_layout: wgpu::BindGroupLayout,

    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl ImguiRenderer {
    /// Creates a new Dear ImGui renderer instance.
    ///
    /// Note, that this doesn't take the Dear ImGui context as a parameter. The context is actually
    /// configured for the renderer as part of [`crate::devui::DevUi`] initialization.
    pub fn new(r: &mut Renderer, format: wgpu::TextureFormat) -> Self {
        let device = &r.dc.device;
        let shader = r.shaders.get("zenit_imgui").expect("missing imgui shader");

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui vertex buffer"),
            size: DEFAULT_BUFFER_SIZE,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui index buffer"),
            size: DEFAULT_BUFFER_SIZE,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("imgui uniform buffer"),
            size: mem::size_of::<[f32; 2]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui bind group layout (for texture mapping)"),
            entries: &bind_group_layout_array![
                0 => (
                    FRAGMENT,
                    BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false
                    },
                ),
                1 => (
                    FRAGMENT,
                    BindingType::Sampler(SamplerBindingType::Filtering),
                )
            ],
        });

        let vertex_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui bind group layout (vertex uniforms)"),
            entries: &bind_group_layout_array![
                0 => (
                    VERTEX,
                    BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(uniform_buffer.size()),
                    }
                )
            ],
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("imgui bind group (vertex uniforms)"),
            layout: &vertex_bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("imgui pipeline"),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("imgui pipeline layout"),
                bind_group_layouts: &[&vertex_bind_layout, &texture_bind_layout],
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: &shader.module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<ImDrawVert>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2, // a_position
                        1 => Float32x2, // a_uv
                        2 => Unorm8x4,  // a_color
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format,
                    // Based on upstream Dear Imgui's Vulkan implementation
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Self {
            textures: AHashMap::with_capacity(10),
            pipeline,
            texture_bind_layout,
            uniform_bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
        }
    }

    /// Renders an ImGui frame, and returns the involved command buffer.
    pub fn render_imgui(
        &mut self,
        dc: &DeviceContext,
        render_data: ImguiRenderData,
        target: &wgpu::TextureView,
    ) -> wgpu::CommandBuffer {
        // Update the uniform buffer
        {
            // eeeeeeeeeeehh
            let width = render_data.display_size.x.to_le_bytes();
            let height = render_data.display_size.y.to_le_bytes();
            dc.queue.write_buffer(
                &self.uniform_buffer,
                0,
                &[
                    width[0], width[1], width[2], width[3], height[0], height[1], height[2],
                    height[3],
                ],
            );
        }

        // Write vertex and index data to the GPU
        {
            let vertex_count = render_data.vertices.len() as u64;
            let total_vertex_size = vertex_count * mem::size_of::<DrawVert>() as u64;
            if self.vertex_buffer.size() < total_vertex_size {
                self.vertex_buffer = dc.device.create_buffer(&BufferDescriptor {
                    label: Some("imgui vertex buffer"),
                    size: total_vertex_size,
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            let index_count = render_data.indices.len() as u64;
            let total_index_size = align(index_count * mem::size_of::<ImDrawIdx>() as u64, 4);
            if self.index_buffer.size() < total_index_size {
                self.index_buffer = dc.device.create_buffer(&BufferDescriptor {
                    label: Some("imgui index buffer"),
                    size: total_index_size,
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            // This is very hacky, but oh my let's go
            // Safety: DrawVert and ImDrawIdx have expected layout
            unsafe {
                dc.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    slice::from_raw_parts(
                        render_data.vertices.as_ptr() as *const u8,
                        render_data.vertices.len() * mem::size_of::<DrawVert>(),
                    ),
                );

                dc.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    slice::from_raw_parts(
                        render_data.indices.as_ptr() as *const u8,
                        align(
                            (render_data.indices.len() * mem::size_of::<ImDrawIdx>()) as u64,
                            4,
                        ) as usize,
                    ),
                );
            }
        }

        // Now we go through all the draw commands
        let mut encoder = dc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("imgui command encoder"),
        });

        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("imgui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        rpass.push_debug_group("imgui render");
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(
            self.index_buffer.slice(..),
            match mem::size_of::<ImDrawIdx>() {
                2 => IndexFormat::Uint16,
                4 => IndexFormat::Uint32,
                _ => unreachable!("invalid index size, expected 2 or 4"),
            },
        );

        for call in render_data.render_calls {
            rpass.set_scissor_rect(
                call.scissor_params.0,
                call.scissor_params.1,
                call.scissor_params.2,
                call.scissor_params.3,
            );

            rpass.set_bind_group(
                1,
                self.textures
                    .get(&call.texture)
                    .expect("invalid texture access in imgui draw")
                    .bind_group
                    .as_ref()
                    .expect("bind group not present, did the render system not generate it?"),
                &[],
            );

            rpass.draw_indexed(call.indices, call.vertex_offset, 0..1);
        }
        rpass.pop_debug_group();

        drop(rpass); // Finish recording the render pass

        encoder.finish()
    }

    /// Performs any tasks that prepare the Dear ImGui renderer for the next frame's render. At
    /// this time, this only includes managing textures.
    pub(in crate::graphics) fn prepare_next_frame(&mut self, renderer: &mut Renderer) {
        if let Some(imgui_frame) = renderer.imgui_frame.as_mut() {
            self.textures.extend(imgui_frame.new_textures.drain(..));
            for to_remove in imgui_frame.textures_to_remove.drain(..) {
                self.textures.remove(&to_remove);
            }
        }

        for (_, texture) in &mut self.textures {
            texture.try_update_bind_group(renderer, &self.texture_bind_layout);
        }
    }
}

/// A standalone structure containing packaged Dear ImGui [`DrawData`] information.
pub struct ImguiRenderData {
    /// The display size in logical coordinates.
    pub display_size: Vec2,
    /// The framebuffer's scale factor
    pub display_scale: Vec2,
    /// The UI mesh's combined vertex buffer
    pub vertices: Vec<DrawVert>,
    /// The UI mesh's combined index buffer
    pub indices: Vec<ImDrawIdx>,
    /// Marks the vertex/index offsets at which each command
    pub render_calls: Vec<ImguiRenderCall>,
}

impl ImguiRenderData {
    pub fn new(draw_data: &DrawData) -> Self {
        // We don't have multi viewport support
        assert!(draw_data.display_pos == [0.0, 0.0]);

        let framebuffer_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let framebuffer_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        let mut vertices = Vec::with_capacity(draw_data.total_vtx_count as usize);
        let mut indices = Vec::with_capacity(draw_data.total_idx_count as usize);
        let mut render_calls = vec![];

        let mut absolute_vertex_offset = 0;
        let mut absolute_index_offset = 0;
        for list in draw_data.draw_lists() {
            vertices.extend(list.vtx_buffer().iter());
            indices.extend(list.idx_buffer().iter());

            for command in list.commands() {
                use imgui::DrawCmd::*;
                match command {
                    Elements { count, cmd_params } => {
                        let DrawCmdParams {
                            mut clip_rect,
                            texture_id,
                            vtx_offset,
                            idx_offset,
                        } = cmd_params;

                        clip_rect[0] *= draw_data.framebuffer_scale[0];
                        clip_rect[1] *= draw_data.framebuffer_scale[1];
                        clip_rect[2] *= draw_data.framebuffer_scale[0];
                        clip_rect[3] *= draw_data.framebuffer_scale[1];
                        let clip_min_x = clip_rect[0].max(0.0).floor();
                        let clip_max_x = clip_rect[2].min(framebuffer_width).floor();
                        let clip_min_y = clip_rect[1].max(0.0).floor();
                        let clip_max_y = clip_rect[3].min(framebuffer_height).floor();
                        if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
                            continue;
                        }

                        let base_index = absolute_index_offset + idx_offset as u32;
                        render_calls.push(ImguiRenderCall {
                            indices: base_index..base_index + count as u32,
                            vertex_offset: absolute_vertex_offset + vtx_offset as i32,
                            texture: texture_id,
                            scissor_params: (
                                (clip_min_x) as u32,
                                (clip_min_y) as u32,
                                (clip_max_x - clip_min_x) as u32,
                                (clip_max_y - clip_min_y) as u32,
                            ),
                        });
                    }
                    ResetRenderState => {}
                    RawCallback { .. } => {
                        panic!("imgui raw callbacks aren't supported")
                    }
                }
            }

            absolute_vertex_offset += list.vtx_buffer().len() as i32;
            absolute_index_offset += list.idx_buffer().len() as u32;
        }

        Self {
            display_size: vec2(draw_data.display_size[0], draw_data.display_size[1]),
            display_scale: vec2(
                draw_data.framebuffer_scale[0],
                draw_data.framebuffer_scale[1],
            ),
            vertices,
            indices,
            render_calls,
        }
    }
}

/// A standalone structure containing a packaged [`imgui::DrawCmd`] draw call
pub struct ImguiRenderCall {
    /// Range of indices within the index buffer
    pub indices: Range<u32>,
    /// Base vertex offset of this call (will be passed as base vertex during the draw call)
    pub vertex_offset: i32,
    /// The ID of the texture to bind
    pub texture: imgui::TextureId,
    /// Scissor parameters for the call
    pub scissor_params: (u32, u32, u32, u32),
}

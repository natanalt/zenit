use std::sync::Arc;
use wgpu::*;
use crate::render::DeviceContext;
use super::Texture2D;
use glam::*;

pub struct Model {
    constant_bind_group: BindGroup,
    textures: [Option<Arc<Texture2D>>; 4],

    /// A buffer providing combined vertex and index data.
    /// Several other fields in this struct proivide offset/size pairs for vertices and indices.
    vertex_index_buffer: Buffer,
    /// Offset at which vertices start in the model's buffer.
    vertex_offset: BufferAddress,
    /// Vertices' size in the model's buffer
    vertex_size: BufferSize,
    /// Offst at which indices start in the model's buffer
    index_offset: BufferAddress,
    /// Indices' size in the model's buffer
    index_size: BufferSize,
}

impl Model {
    pub fn create_textured_plane(
        dc: &DeviceContext,
        texture: &Texture2D,
        extents: UVec2,
    ) -> Self {
        todo!()
    }
}

pub struct ModelInstance {
    model: Arc<Model>,
    uniform_bind_group: BindGroup,
}

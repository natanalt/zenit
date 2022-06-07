//! Various helpers for rendering

/// [`wgpu::PrimitiveState`] that's most likely to reappear in various
/// pipelines.
pub const USUAL_PRIMITIVES: wgpu::PrimitiveState = wgpu::PrimitiveState {
    topology: wgpu::PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Cw,
    cull_mode: Some(wgpu::Face::Back),
    unclipped_depth: false,
    polygon_mode: wgpu::PolygonMode::Fill,
    conservative: false,
};

/// Generates a [`wgpu::VertexBufferLayout`] that describes a vertex buffer
/// made of a single, tightly packed buffer, with vertex step mode.
/// 
/// Takes the same parameters as the [`wgpu::vertex_attr_array`] macro.
/// 
/// ## Example
/// ```rs
/// // To be passed in vertex stage description while creating a pipeline
/// let buffers = &[crate::single_vertex_buffer![
///     0 => Float32x2,
///     1 => Float32x4,
/// ]];
/// ```
#[macro_export]
macro_rules! single_vertex_buffer {
    ($($vertices:tt)*) => {
        {
            const ATTRIBS: &'static [::wgpu::VertexAttribute] = &::wgpu::vertex_attr_array![
                $($vertices)*
            ];

            let size = ATTRIBS.last()
                .map(|attr| attr.offset + attr.format.size())
                .unwrap_or(0);

            ::wgpu::VertexBufferLayout {
                array_stride: size,
                step_mode: ::wgpu::VertexStepMode::Vertex,
                attributes: ATTRIBS,
            }
        }
    }
}

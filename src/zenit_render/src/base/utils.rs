
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

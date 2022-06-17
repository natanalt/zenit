use crevice::std140::AsStd140;

/// Shared camera uniform, GLSL defined in `assets/shaders/shared_camera.inc`
#[derive(Debug, Clone, AsStd140)]
pub struct CameraUniform {
    projection: glam::Mat4,
    world_to_view: glam::Mat4,
}

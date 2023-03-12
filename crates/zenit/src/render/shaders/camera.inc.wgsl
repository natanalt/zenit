
// Keep this uniform buffer in sync with render/resources/camera.rs
struct CameraBuffer {
    projection: mat4x4<f32>,
    world_to_view: mat4x4<f32>,
    texture_size: vec2<u32>,
}

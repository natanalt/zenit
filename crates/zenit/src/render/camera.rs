use std::sync::Arc;
use parking_lot::Mutex;
use zenit_utils::{math::Radians, ArcPoolHandle};
use crate::ecs::{Component, Entity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraHandle(pub(super) ArcPoolHandle);

#[derive(Clone)]
pub struct CameraResource {
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,

    pub(super) gpu_resources: Arc<Mutex<CameraGpuResources>>,
}

pub(super) struct CameraGpuResources {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub buffer: wgpu::Buffer,
}

// TODO: what happens if multiple CameraComponents link to the same camera resource?
// TODO: what happens if the render_to_screen camera has a size different from the framebuffer?
// Neither of these scenarios should happen in normal gameplay, but they still can safely happen

pub struct CameraComponent {
    /// If true, this camera will be rendered to.
    pub enabled: bool,
    /// If true, this camera will be rendered on the screen. Only one enabled camera can render
    /// to the screen at once.
    pub render_to_screen: bool,
    /// Underlying camera resource to render to
    pub camera_handle: CameraHandle,
    /// Target entity with a [`super::SceneComponent`].
    pub scene_entity: Entity,
}

impl Component for CameraComponent {}

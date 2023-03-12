use std::sync::Arc;
use crate::{render::resources::Camera, ecs::{Component, Entity}};

// TODO: what happens if multiple CameraComponents link to the same camera resource?
// TODO: what happens if the render_to_screen camera has a size different from the framebuffer?

pub struct CameraComponent {
    /// If true, this camera will be rendered on the screen. Only one enabled camera can render
    /// to the screen at once.
    pub render_to_screen: bool,
    /// If true, this camera will be rendered to.
    pub enabled: bool,
    /// Underlying camera resource to render to
    pub camera_handle: Arc<Camera>,
    /// Target entity with a [`super::SceneComponent`].
    pub target_scene: Entity,
}

impl Component for CameraComponent {}

impl AsRef<Camera> for CameraComponent {
    fn as_ref(&self) -> &Camera {
        &self.camera_handle
    }
}
use std::sync::Arc;
use derive_builder::Builder;
use glam::{IVec2, Vec3A, Quat};
use wgpu::TextureFormat;
use zenit_utils::math::Radians;
use super::Scenario;

/// A camera specifies from what point a scene is rendered.
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Camera {
    pub position: Vec3A,
    pub rotation: Quat,
    pub fov: Radians,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn builder() -> CameraBuilder {
        CameraBuilder::default()
    }
}

/// A viewport combines [`Scenario`], [`Camera`] rendering the scene to an
/// owned texture.
pub struct Viewport {
    pub camera: Camera,
    pub view: wgpu::TextureView,
    pub scenario: Arc<Scenario>,
}

#[derive(Builder)]
pub struct ViewportCreationInfo {
    pub resolution: IVec2,
    pub format: TextureFormat,
}

impl ViewportCreationInfo {
    pub fn builder() -> ViewportCreationInfoBuilder {
        ViewportCreationInfoBuilder::default()
    }
}

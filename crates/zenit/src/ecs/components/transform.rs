use glam::{Affine3A, Vec3A, Mat4};
use crate::ecs::Component;

/// Represents a 3D entity transform, with a translation and any 3D transformation.
#[derive(Debug, Clone)]
pub struct TransformComponent(pub Affine3A);

impl Component for TransformComponent {}

impl TransformComponent {
    /// Returns the transform's translation.
    #[inline]
    pub fn translation(&self) -> Vec3A {
        self.0.translation
    }

    /// Converts the internal transform into a [`Mat4`].
    #[inline]
    pub fn as_mat4(&self) -> Mat4 {
        self.0.into()
    }

    /// Adds the provided vector to this transform's translation.
    #[inline]
    pub fn translate(&mut self, v: Vec3A) {
        self.0.translation += v;
    }
}

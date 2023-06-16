use crate::entities::Component;
use glam::*;

/// Represents a 3D entity transform, with a translation and any 3D transformation that can be
/// represented by a 3x3 matrix.
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

    /// Extracts the rotation out of this transform.
    #[inline]
    pub fn rotation(&self) -> Quat {
        Quat::from_affine3(&self.0)
    }

    /// Adds the provided vector to this transform's translation.
    #[inline]
    pub fn translate(&mut self, v: Vec3A) {
        self.0.translation += v;
    }
}

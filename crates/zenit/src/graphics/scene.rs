use super::SkyboxHandle;
use crate::entities::{Component, Entity};

/// Defines a scene and its basic parameters. Entities with [`RenderComponent`]s link to an entity
/// with this component.
pub struct SceneComponent {
    pub skybox: Option<SkyboxHandle>,
}

impl Component for SceneComponent {}

/// Marks the entity as containing potential render items for a scene.
pub struct RenderComponent {
    /// Parent entity with a [`SceneComponent`].
    pub parent: Entity,
}

impl Component for RenderComponent {}

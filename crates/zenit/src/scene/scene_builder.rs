use crate::{
    entities::{components::TransformComponent, Entity, Universe},
    graphics::{CameraComponent, CameraHandle, SceneComponent, SkyboxHandle},
};
use glam::Affine3A;
use thiserror::Error;

/// Error used by [`SceneBuilder::from_scene`].
#[derive(Debug, Error)]
#[error("specified entity doesn't constitute a valid scene")]
pub struct InvalidSceneError;

/// The scene builder is a simplified interface for creating scenes and appending render entities
/// into them. It's meant to be the preferred way of creating scenes, as compared to manually
/// creating and configuring entities.
pub struct SceneBuilder<'a> {
    universe: &'a mut Universe,
    scene: Entity,
}

impl<'a> SceneBuilder<'a> {
    /// Creates a new scene for a new builder.
    pub fn new(universe: &'a mut Universe) -> Self {
        let scene = universe.create_entity_with(SceneComponent::default());
        Self { universe, scene }
    }

    pub fn from_scene(
        universe: &'a mut Universe,
        entity: Entity,
    ) -> Result<Self, InvalidSceneError> {
        if universe.has_component::<SceneComponent>(entity) {
            Ok(Self {
                universe,
                scene: entity,
            })
        } else {
            Err(InvalidSceneError)
        }
    }

    pub fn create_camera(&mut self, transform: Affine3A, handle: CameraHandle) -> Entity {
        self.universe
            .build_entity()
            .with_component(TransformComponent(transform))
            .with_component(CameraComponent {
                enabled: true,
                camera_handle: handle,
                scene_entity: self.scene,
            })
            .finish()
    }

    pub fn set_skybox(&mut self, skybox: SkyboxHandle) {
        self.universe
            .get_component_mut::<SceneComponent>(self.scene)
            .expect("scene component has gone missing")
            .skybox = Some(skybox);
    }

    pub fn finish(self) -> Entity {
        self.scene
    }
}

use super::{
    components::{CameraComponent, RenderComponent, SceneComponent},
    resources::{Camera, Skybox},
};
use crate::ecs::{accessor::EntityAccessor, Entity, Universe};
use ahash::AHashMap;
use std::sync::Arc;

/// Contains all render information for the next frame.
#[derive(Default)]
pub struct FrameState {
    pub screen_target: Option<Arc<Camera>>,
    pub targets: Vec<(Arc<Camera>, Arc<FrameScene>)>,
}

impl FrameState {
    /// Gathers frame state information from the ECS. It's a rather expensive operation,
    /// which is done only once per frame.
    pub fn from_ecs(universe: &Universe) -> Self {
        let mut screen_target = None;

        let mut scenes: AHashMap<Entity, FrameScene> = universe
            .get_components::<SceneComponent>()
            .map(|(entity, scene)| {
                (
                    entity,
                    FrameScene::from_entity(universe.access_entity(entity), scene),
                )
            })
            .collect();

        for entity in universe.iter_entities() {
            if !entity.has_component::<RenderComponent>() {
                continue;
            }

            // TODO: process the render entity
        }

        let scenes_immutable: AHashMap<Entity, Arc<FrameScene>> = scenes
            .into_iter()
            .map(|(entity, scene)| (entity, Arc::new(scene)))
            .collect();

        let targets = universe
            .get_components::<CameraComponent>()
            .filter(|(_, camera)| camera.enabled)
            .map(|(entity, camera)| {
                if camera.render_to_screen {
                    assert!(screen_target.is_none(), "multiple screen target cameras are enabled");
                    screen_target = Some(camera.camera_handle.clone());
                }

                (
                    camera.camera_handle.clone(),
                    scenes_immutable
                        .get(&entity)
                        .expect("invalid scene reference in a camera component")
                        .clone(),
                )
            })
            .collect();

        Self { screen_target, targets }
    }
}

pub struct FrameScene {
    pub parent_entity: Entity,
    pub skybox: Arc<Skybox>,
}

impl FrameScene {
    pub fn from_entity(entity: EntityAccessor, scene: &SceneComponent) -> Self {
        Self {
            parent_entity: entity.entity(),
            skybox: scene.skybox.clone(),
        }
    }
}

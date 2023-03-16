use super::{
    CameraComponent, CameraGpuResources, RenderComponent, Renderer, SceneComponent,
};
use crate::ecs::{accessor::EntityAccessor, components::TransformComponent, Entity, Universe};
use ahash::AHashMap;
use glam::*;
use parking_lot::Mutex;
use std::sync::Arc;
use zenit_utils::math::Radians;

/// Standalone\* frame snapshot that can be used by the render system.
/// 
/// \* Well, *technically* it includes shared references to GPU resources, but those aren't
/// to be exposed to main game code anyway.
pub struct FrameState {
    /// Handle to the camera that'll be rendered on the screen.
    pub screen_target: Option<CameraState>,
    /// Linear list of cameras and scenes they should render
    pub targets: Vec<(CameraState, Arc<FrameScene>)>,
}

impl FrameState {
    /// Collects a frame snapshot from the ECS.
    ///
    /// This is a slightly expensive operation, done once a frame.
    pub fn from_ecs(universe: &Universe, renderer: &Renderer) -> Self {
        let mut screen_target = None;

        // Temporary storage of scenes, mapping each scene entity to its FrameScene instance
        let mut scene_map: AHashMap<Entity, Arc<FrameScene>> = universe
            .get_components::<SceneComponent>()
            .map(|(accessor, sc)| {
                (
                    accessor.entity(),
                    Arc::new(FrameScene::from_ecs(accessor.entity(), sc)),
                )
            })
            .collect();

        // Since scene_map uses Arcs, we unwrap them here to mutable references
        let mut scene_refs: AHashMap<Entity, &mut FrameScene> = scene_map
            .iter_mut()
            .map(|(&entity, fs)| (entity, Arc::get_mut(fs).unwrap()))
            .collect();

        // Scan each render entity
        for (entity, rc) in universe.get_components::<RenderComponent>() {
            let scene = scene_refs
                .get(&rc.parent)
                .expect("render entity has an invalid scene entity");

        }

        // Map each camera to its Arc<FrameScene>
        let targets = universe
            .get_components::<CameraComponent>()
            .filter(|(_, cc)| cc.enabled)
            .map(|(accessor, cc)| {
                let camera_state = CameraState::from_ecs(accessor, renderer);

                if cc.render_to_screen {
                    assert!(
                        screen_target.is_none(),
                        "multiple render to screen cameras are active"
                    );
                    screen_target = Some(camera_state.clone());
                }

                (
                    camera_state,
                    scene_map
                        .get(&cc.scene_entity)
                        .expect("camera has an invalid scene entity handle")
                        .clone(),
                )
            })
            .collect();

        Self {
            screen_target,
            targets,
        }
    }
}

/// Static snapshot of a camera.
#[derive(Clone)]
pub struct CameraState {
    pub position: Vec3A,
    pub rotation: Quat,
    pub near_plane: f32,
    pub far_plane: f32,
    pub fov: Radians,
    pub(super) gpu_resources: Arc<Mutex<CameraGpuResources>>,
}

impl CameraState {
    pub fn from_ecs(entity: EntityAccessor, renderer: &Renderer) -> Self {
        let transform: &TransformComponent =
            entity.get_component().expect("missing TransformComponent");
        let camera_component: &CameraComponent =
            entity.get_component().expect("missing CameraComponent");
        let camera = renderer.get_camera(&camera_component.camera_handle);

        Self {
            position: transform.translation(),
            rotation: transform.rotation(),
            near_plane: camera.near_plane,
            far_plane: camera.far_plane,
            fov: camera.fov,
            gpu_resources: camera.gpu_resources.clone(),
        }
    }
}

/// Snapshot of a single scene.
pub struct FrameScene {
    pub parent_entity: Entity,
    // todo...
}

impl FrameScene {
    pub fn from_ecs(entity: Entity, scene: &SceneComponent) -> Self {
        Self {
            parent_entity: entity,
        }
    }
}

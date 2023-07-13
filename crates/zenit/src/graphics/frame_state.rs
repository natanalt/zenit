use super::{
    CameraComponent, CameraGpuResources, RenderComponent, Renderer, SceneComponent,
    SkyboxGpuResources, imgui_renderer::ImguiRenderData,
};
use crate::entities::{components::TransformComponent, Entity, EntityAccessor, Universe};
use ahash::AHashMap;
use glam::*;
use parking_lot::Mutex;
use std::sync::Arc;
use zenit_utils::math::Radians;

/// Standalone[1] frame snapshot that can be used by the render system.
///
/// [1] Well, *technically* it includes shared references to GPU resources, but those aren't
/// to be exposed to main game code anyway.
pub struct FrameState {
    /// Linear list of cameras and scenes they should render
    pub targets: Vec<(CameraState, Arc<FrameScene>)>,
    /// The ImGui frame information. This is what is rendered on the window if present.
    pub imgui_frame: Option<ImguiRenderData>,
}

impl FrameState {
    /// Collects a frame snapshot from the ECS.
    ///
    /// This is a slightly expensive operation, done once a frame.
    pub fn collect_state(universe: &Universe, renderer: &mut Renderer) -> Self {

        // Temporary storage of scenes, mapping each scene entity to its FrameScene instance
        let mut scene_map: AHashMap<Entity, Arc<FrameScene>> = universe
            .get_components::<SceneComponent>()
            .map(|(accessor, sc)| {
                (
                    accessor.entity(),
                    Arc::new(FrameScene::from_ecs(accessor.entity(), sc, renderer)),
                )
            })
            .collect();

        // Since scene_map uses Arcs, we unwrap them here to mutable references
        let scene_refs: AHashMap<Entity, &mut FrameScene> = scene_map
            .iter_mut()
            .map(|(&entity, fs)| (entity, Arc::get_mut(fs).unwrap()))
            .collect();

        // Scan each render entity
        for (_entity, rc) in universe.get_components::<RenderComponent>() {
            let _scene = scene_refs
                .get(&rc.parent)
                .expect("render entity has an invalid scene entity");
        }

        // Map each camera to its Arc<FrameScene>
        let targets = universe
            .get_components::<CameraComponent>()
            .filter(|(_, cc)| cc.enabled)
            .map(|(accessor, cc)| {
                (
                    CameraState::from_ecs(accessor, renderer),
                    scene_map
                        .get(&cc.scene_entity)
                        .expect("camera has an invalid scene entity handle")
                        .clone(),
                )
            })
            .collect();

        Self {
            targets,
            imgui_frame: renderer.imgui_frame.take().map(|frame| frame.draw_data),
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
    pub(in crate::graphics) gpu_resources: Arc<Mutex<CameraGpuResources>>,
}

impl CameraState {
    pub fn from_ecs(entity: EntityAccessor, renderer: &Renderer) -> Self {
        let transform: &TransformComponent = entity
            .get_component()
            .expect("missing TransformComponent in a camera");
        let camera_component: &CameraComponent = entity
            .get_component()
            .expect("missing CameraComponent in a camera");
        let camera = renderer.cameras.get(&camera_component.camera_handle);

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
    pub(in crate::graphics) skybox_resources: Option<Arc<Mutex<SkyboxGpuResources>>>,
}

impl FrameScene {
    pub fn from_ecs(entity: Entity, scene: &SceneComponent, renderer: &Renderer) -> Self {
        Self {
            parent_entity: entity,
            skybox_resources: scene
                .skybox
                .as_ref()
                .and_then(|skybox| Some(renderer.skyboxes.get(skybox).gpu_resources.clone())),
        }
    }
}

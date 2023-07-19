use super::{CameraGpuResources, CameraHandle, Renderer, SkyboxGpuResources, SkyboxHandle};
use glam::Affine3A;
use parking_lot::Mutex;
use std::sync::Arc;

/// Immediate-style builder for scenes that can be submitted to the renderer
pub struct SceneBuilder<'a> {
    renderer: &'a mut Renderer,
    pub(in crate::graphics) built: BuiltScene,
}

impl<'a> SceneBuilder<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self {
            renderer,
            built: BuiltScene::default(),
        }
    }

    pub fn set_skybox(&mut self, skybox: &SkyboxHandle) -> &mut Self {
        self.built.skybox = Some(self.renderer.skyboxes.get(skybox).gpu_resources.clone());
        self
    }

    pub fn render_to(&mut self, camera: &CameraHandle, transform: Affine3A) -> &mut Self {
        self.built.targets.push((
            self.renderer.cameras.get(camera).gpu_resources.clone(),
            transform,
        ));
        self
    }

    pub fn submit(self) {
        self.renderer.pending_frame.scenes.push(self.built);
    }
}

#[derive(Default)]
pub(in crate::graphics) struct BuiltScene {
    pub targets: Vec<(Arc<Mutex<CameraGpuResources>>, Affine3A)>,
    pub skybox: Option<Arc<Mutex<SkyboxGpuResources>>>,
}

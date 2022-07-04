use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, StageLabel)]
pub enum SceneRenderStage {
    Skybox,
    Opaque,
    Transparent,
}

pub fn create_scene_scheduler() -> Schedule {
    let mut s = Schedule::default();
    s.add_stage(SceneRenderStage::Skybox, SystemStage::parallel());
    s.add_stage(SceneRenderStage::Opaque, SystemStage::parallel());
    s.add_stage(SceneRenderStage::Transparent, SystemStage::parallel());
    s
}

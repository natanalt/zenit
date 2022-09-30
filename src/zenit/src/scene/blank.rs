use super::{
    ecs::{EntityBehavior, EntityStorage},
    SceneState,
};

pub struct NoBehavior;

impl EntityBehavior for NoBehavior {
    fn process(&mut self, entity: &mut EntityStorage, scene: &mut SceneState) {
        let _ = entity;
        let _ = scene;
    }
}

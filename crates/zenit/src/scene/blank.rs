use super::{EngineState, EntityBehavior, EntityStorage};

pub struct NoBehavior;

impl EntityBehavior for NoBehavior {
    fn process(&mut self, entity: &mut EntityStorage, scene: EngineState) {
        let _ = entity;
        let _ = scene;
    }
}

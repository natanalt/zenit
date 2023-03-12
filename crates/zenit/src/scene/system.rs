use std::sync::Arc;
use parking_lot::RwLock;
use crate::{engine::{System, EngineContext, GlobalContext}, ecs::Universe};

pub struct SceneSystem {
    universe: Arc<RwLock<Universe>>,
}

impl SceneSystem {
    pub fn new() -> Self {
        Self {
            universe: Arc::new(RwLock::new(Universe::new())),
        }
    }
}

impl System for SceneSystem {
    fn label(&self) -> &'static str {
        "Scene System"
    }
    
    fn init(&mut self, ec: &mut EngineContext) {
        let gc = ec.global_context.get_mut();
        gc.universe = Some(self.universe.clone());

        let mut u = self.universe.write();
        println!("craeting a universe");
        dbg!(u.create_entity());
        dbg!(u.create_entity());
        dbg!(u.create_entity());
        dbg!(u.create_entity());
    }

    fn frame_initialization(&mut self, ec: &EngineContext) {
        let _ = ec;
    }

    fn main_process(&mut self, ec: &EngineContext, gc: &GlobalContext) {
        let _ = (ec, gc);
    }

    fn post_process(&mut self, ec: &EngineContext) {
        let _ = ec;
    }
}


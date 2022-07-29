//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use zenit_proc::HasSystemInterface;
use crate::engine::system::{System, SystemContext};

#[derive(Default, HasSystemInterface)]
pub struct SceneSystem;

impl System for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn frame(&mut self, context: SystemContext) {
        //todo!()
    }
}


//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use zenit_proc::HasSystemInterface;
use crate::{engine::system::{System, SystemContext}, cli};

#[derive(Default, HasSystemInterface)]
pub struct SceneSystem;

impl<'ctx> System<'ctx> for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn init(&mut self, context: &SystemContext<'ctx>) {
        let cli = context.data::<cli::Args>();
        log::debug!("cli = {cli:#?}");
    }

    fn frame(&mut self, _context: &SystemContext<'ctx>) {
        
    }
}

//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use crate::{
    cli,
    engine::system::{System, SystemContext},
};
use zenit_proc::HasSystemInterface;

pub mod ecs;

#[derive(Default, HasSystemInterface)]
pub struct SceneSystem;

impl<'ctx> System<'ctx> for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn init(&mut self, context: &mut SystemContext<'ctx>) {
        let cli = context.data::<cli::Args>();
        log::debug!("cli = {cli:#?}");
    }

    fn frame(&mut self, _context: &mut SystemContext<'ctx>) {
        //if context.events.len() > 0 {
        //    log::debug!("got some events");
        //    log::debug!("{:#?}", context.events[0]);
        //}
    }
}

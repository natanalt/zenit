//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use crate::{
    cli,
    engine::system::{System, SystemContext},
};
use std::{
    cell::{RefCell, RefMut},
    iter,
    num::NonZeroUsize,
};
use thiserror::Error;
use zenit_proc::HasSystemInterface;
use zenit_utils::ThreadCell;

pub mod ecs;

#[derive(HasSystemInterface)]
pub struct SceneSystem {
    //root: ThreadCell<NodeRef<BlankNode>>,
}

impl Default for SceneSystem {
    fn default() -> Self {
        Self {}
        //Self { root: ThreadCell::new(BlankNode::new()) }
    }
}

impl<'ctx> System<'ctx> for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn init(&mut self, context: &mut SystemContext<'ctx>) {
        let cli = context.data::<cli::Args>();
        log::debug!("cli = {cli:#?}");
    }

    fn frame(&mut self, _context: &mut SystemContext<'ctx>) {
        //let mut root = self.root.get_mut().unwrap();
        //root.borrow_imp().process(&root.0.)
        todo!()
    }
}

pub struct SceneState {
    
}

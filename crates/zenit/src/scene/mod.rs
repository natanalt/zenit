use crate::{assets::AssetManager, engine::GlobalState, entities::Universe, graphics::Renderer};

pub mod system;

pub struct EngineBorrow<'a> {
    pub globals: &'a GlobalState,
    pub assets: &'a mut AssetManager,
    pub renderer: &'a mut Renderer,
    pub universe: &'a mut Universe,
}

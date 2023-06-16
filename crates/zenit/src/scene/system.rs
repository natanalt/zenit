use crate::{
    assets::{AssetLoader, AssetManager, GameRoot},
    devui::DevUi,
    engine::{EngineContext, GlobalState, System},
    entities::Universe,
    graphics::Renderer,
    scene::EngineBorrow,
};
use log::*;
use std::sync::Arc;
use winit::window::Window;

pub struct SceneSystem {
    devui: DevUi,
}

impl SceneSystem {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            devui: DevUi::new(&window),
        }
    }
}

impl System for SceneSystem {
    fn label(&self) -> &'static str {
        "Scene System"
    }

    fn init(&mut self, ec: &mut EngineContext) {
        let gc = ec.globals.get_mut();
        let assets = AssetManager::new(gc.get::<GameRoot>().clone(), &mut gc.lock::<Renderer>());
        gc.add_lockable(assets);
        gc.add_rw_lockable(Universe::new());

        let mut assets = gc.lock::<AssetManager>();
        let mut renderer = gc.lock::<Renderer>();
        let mut universe = gc.write::<Universe>();

        trace!("Initializing the global asset manager...");
        AssetLoader::new(&mut EngineBorrow {
            globals: &gc,
            assets: &mut assets,
            renderer: &mut renderer,
            universe: &mut universe,
        })
        .load_builtins()
        .expect("could not load built-in assets");
    }

    fn frame_initialization(&mut self, ec: &EngineContext) {
        let _ = ec;
    }

    fn main_process(&mut self, _ec: &EngineContext, gs: &GlobalState) {
        let mut universe = gs.write::<Universe>();
        let mut renderer = gs.lock::<Renderer>();
        let mut assets = gs.lock::<AssetManager>();

        let mut engine = EngineBorrow {
            globals: gs,
            assets: &mut assets,
            renderer: &mut renderer,
            universe: &mut universe,
        };

        self.devui.process(&mut engine);
    }

    fn post_process(&mut self, ec: &EngineContext) {
        let _ = ec;
    }
}

use crate::{
    assets::AssetManager,
    devui::DevUi,
    engine::{EngineContext, GlobalState, System},
    entities::Universe,
    graphics::Renderer,
    scene::EngineBorrow,
};
use zenit_utils::ThreadCell;

pub struct SceneSystem {
    devui: Option<ThreadCell<DevUi>>,
}

impl SceneSystem {
    pub fn new() -> Self {
        Self {
            devui: None,
        }
    }
}

impl System for SceneSystem {
    fn label(&self) -> &'static str {
        "Scene System"
    }

    fn first_frame(&mut self, ec: &EngineContext) {
        // We initialize the DevUi on the scene system thread due to imgui-rs constraints
        self.devui = Some(ThreadCell::new(DevUi::new(ec)));
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

        if let Some(devui) = self.devui.as_mut() {
            devui.get_mut().process(&mut engine);
        }
    }
}

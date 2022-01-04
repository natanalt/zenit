use log::info;
use winit::window::Window;

pub struct Renderer {

}

impl Renderer {
    pub async fn new(_window: &Window) -> Self {
        info!("Initializing the renderer...");
        Renderer {}
    }
}

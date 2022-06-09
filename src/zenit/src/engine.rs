use crate::render::Renderer;
use std::{sync::Arc, time::Duration};
use winit::window::Window;

pub struct Engine {
    pub window: Arc<Window>,
    pub renderer: Renderer,
}

pub struct FrameInfo {
    pub delta: Duration,
    pub frame_count: u64,
}

impl FrameInfo {
    pub fn is_on_first_frame(&self) -> bool {
        self.frame_count == 0
    }
}

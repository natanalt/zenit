use crate::{profiling::FrameProfiler, render::Renderer, root::GameRoot};
use std::{sync::Arc, time::Duration};
use winit::window::Window;

pub struct Engine {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub frame_profiler: FrameProfiler,
    pub game_root: Option<GameRoot>,
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

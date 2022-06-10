use std::{collections::VecDeque, time::Duration};

pub struct FrameProfiler {
    pub max_frames: usize,
    pub frames: VecDeque<FrameEntry>,

    pub max_time: Duration,
    pub min_time: Duration,

    running_sum: Duration,
    running_frames: u32,
    pub avg_time: Duration,

    // UI stuff
    pub ui_color: egui::Color32,
    pub game_color: egui::Color32,
    pub render_color: egui::Color32,
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            max_frames: 1000,
            frames: VecDeque::new(),

            max_time: Duration::ZERO,
            min_time: Duration::MAX,

            running_sum: Duration::ZERO,
            running_frames: 0,
            avg_time: Duration::ZERO,

            ui_color: egui::Color32::GREEN,
            game_color: egui::Color32::RED,
            render_color: egui::Color32::BLUE,
        }
    }

    pub fn push_frame(&mut self, frame: FrameEntry) {
        let total = frame.total_time();
        self.max_time = self.max_time.max(total);
        self.min_time = self.min_time.min(total);

        self.running_sum += total;
        self.running_frames += 1;
        self.avg_time = self.running_sum / self.running_frames;

        if self.frames.len() == self.max_frames {
            self.frames.pop_back();
        }
        self.frames.push_front(frame);
    }

    pub fn reset(&mut self) {
        self.frames.clear();
        self.max_time = Duration::ZERO;
        self.min_time = Duration::MAX;
        self.avg_time = Duration::ZERO;
        self.running_sum = Duration::ZERO;
        self.running_frames = 0;
    }
}

#[derive(Default, Clone)]
pub struct FrameEntry {
    pub ui_time: Duration,
    pub game_time: Duration,
    pub render_time: Duration,
}

impl FrameEntry {
    pub fn total_time(&self) -> Duration {
        self.ui_time + self.game_time + self.render_time
    }
}

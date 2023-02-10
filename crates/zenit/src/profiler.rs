use std::{
    mem,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStage {
    // 🏳️‍⚧️
    Transition,
    EventProcessing,
    SceneProcessing,
    RenderProcessing,
}

pub struct FrameProfiler {
    frame_start: Instant,
    segments: Vec<(FrameStage, Duration)>,
    current_stage_start: Instant,
    current_stage: FrameStage,
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            frame_start: Instant::now(),
            segments: vec![],
            current_stage_start: Instant::now(),
            current_stage: FrameStage::Transition,
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
        self.segments.clear();
        self.current_stage_start = self.frame_start;
        self.current_stage = FrameStage::EventProcessing;
    }

    pub fn next_stage(&mut self, stage: FrameStage) {
        self.segments.push((
            self.current_stage,
            Instant::now().duration_since(self.current_stage_start),
        ));
        self.current_stage_start = Instant::now();
        self.current_stage = stage;
    }

    pub fn finish_stage(&mut self) {
        self.next_stage(FrameStage::Transition);
    }

    pub fn finish_frame(&mut self) -> ProfilingInfo {
        self.finish_stage();
        ProfilingInfo {
            total_time: Instant::now().duration_since(self.current_stage_start),
            segments: mem::replace(&mut self.segments, Vec::with_capacity(8)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfilingInfo {
    pub total_time: Duration,
    pub segments: Vec<(FrameStage, Duration)>,
}

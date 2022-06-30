use bevy_ecs::prelude::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use zenit_proc::TupledContainerDerefs;

use crate::schedule::TopFrameStage;

// TODO: improve the frame profiler
//
// Zenit could use custom system executors to track execution times of
// individual systems to display them in some form within the engine.
//
// It's a medium-length idea to undertake, and I've no idea whether
// it can be actually worth it

pub fn init(world: &mut World, schedule: &mut Schedule) {
    // TODO: properly determine refresh rates
    let target_fps = 60;

    world.insert_resource(Delta(Duration::from_secs_f32(1.0 / target_fps as f32)));
    world.insert_resource(FrameProfiler {
        target_fps,
        fps: target_fps,
        frames_this_second: 0,
        second_counter: Duration::ZERO,

        max_frames: 1500,
        frame_times: VecDeque::new(),

        max_time: Duration::ZERO,
        min_time: Duration::MAX,

        running_sum: Duration::ZERO,
        running_frames: 0,
        avg_time: Duration::ZERO,

        frame_start: None,
    });

    schedule.add_system_to_stage(
        TopFrameStage::PreFrameStart,
        move |mut profiler: ResMut<FrameProfiler>| {
            assert!(
                profiler.frame_start.is_none(),
                "frame profiler is still counting a frame",
            );
            profiler.frame_start = Some(Instant::now());
        },
    );

    schedule.add_system_to_stage(
        TopFrameStage::PostFrameFinish,
        move |mut profiler: ResMut<FrameProfiler>, mut delta: ResMut<Delta>| {
            let frame_start = profiler
                .frame_start
                .take()
                .expect("frame profiler is not counting a frame");
            let frame_end = Instant::now();
            let frame_time = frame_end.duration_since(frame_start);
            profiler.push_frame(frame_time);
            *delta = Delta(frame_time);
        },
    );
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, TupledContainerDerefs)]
pub struct Delta(pub Duration);

impl Into<f32> for Delta {
    #[inline]
    fn into(self) -> f32 {
        self.as_secs_f32()
    }
}

pub struct FrameProfiler {
    /// Basically meant to be the refresh rate.
    /// Currently it's displayed on the frame time graph as the high value
    pub target_fps: u32,
    pub fps: u32,
    frames_this_second: u32,
    second_counter: Duration,

    pub max_frames: usize,
    pub frame_times: VecDeque<Duration>,

    pub max_time: Duration,
    pub min_time: Duration,

    running_sum: Duration,
    running_frames: u32,
    pub avg_time: Duration,

    frame_start: Option<Instant>,
}

impl FrameProfiler {
    pub fn push_frame(&mut self, total: Duration) {
        self.frames_this_second += 1;
        self.second_counter += total;
        if self.second_counter.as_secs_f32() >= 1.0 {
            self.fps = self.frames_this_second;
            self.frames_this_second = 0;
            self.second_counter = Duration::ZERO;
        }

        self.max_time = self.max_time.max(total);
        self.min_time = self.min_time.min(total);

        self.running_sum += total;
        self.running_frames += 1;
        self.avg_time = self.running_sum / self.running_frames;

        if self.frame_times.len() == self.max_frames {
            self.frame_times.pop_back();
        }
        self.frame_times.push_front(total);
    }

    /// Calculates maximum duration for given target FPS
    pub fn max_frame_time(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.target_fps as f32)
    }

    /// Clears all tracked values to their default values
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.max_time = Duration::ZERO;
        self.min_time = Duration::MAX;
        self.avg_time = Duration::ZERO;
        self.running_sum = Duration::ZERO;
        self.running_frames = 0;
    }
}

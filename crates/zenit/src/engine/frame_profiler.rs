use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;
use std::{
    collections::VecDeque,
    fmt::Debug,
    mem,
    sync::Arc,
    time::{Duration, Instant},
};

pub type FrameHistory = Arc<RwLock<VecDeque<FrameTiming>>>;

pub(in crate::engine) struct FrameProfiler {
    pub max_history_size: usize,
    pub history: FrameHistory,
    profilers: Vec<Arc<Mutex<SystemProfiler>>>,
    pending: FrameTiming,
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            // Should amount to ~80 seconds @ 60 FPS, ~34 seconds @ 144 FPS
            // This could be made reconfigurable via cmdline args, or config or something.
            // This should amount to up to ~2,17 MiB of frame profiler logs
            max_history_size: 5000,
            history: FrameHistory::default(),
            profilers: vec![],
            pending: FrameTiming::default(),
        }
    }

    pub fn add_system(&mut self, label: &'static str) -> Arc<Mutex<SystemProfiler>> {
        let profiler = Arc::new(Mutex::new(SystemProfiler {
            label,
            frame_init_start: None,
            frame_init_end: None,
            main_process_start: None,
            main_process_end: None,
            post_process_start: None,
            post_process_end: None,
        }));

        self.profilers.push(profiler.clone());
        profiler
    }

    pub fn begin_frame(&mut self) {
        debug_assert!(self.pending.controller_start.is_none());
        self.pending.controller_start = Some(Instant::now());
    }

    pub fn finish_frame(&mut self) {
        self.pending.controller_end = Some(Instant::now());

        let mut pending = mem::take(&mut self.pending);
        for profiler in &self.profilers {
            let mut sp = profiler.lock();
            pending.system_timings.push(sp.reset());
        }

        let mut history = self.history.write();
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(pending);
    }
}

#[derive(Default, Clone)]
pub struct FrameTiming {
    /// Moment when the engine controller itself began frame processing
    pub controller_start: Option<Instant>,
    /// Moment when the engine controller itself finished frame processing.
    pub controller_end: Option<Instant>,
    /// Individual systems' timing information.
    pub system_timings: SmallVec<[SystemTiming; 5]>,
}

impl FrameTiming {
    /// The time spent in the frame by the controller; the most accurate thing you can get to a
    /// *"delta time"* or whatever.
    #[doc(alias = "delta")]
    pub fn controller_time(&self) -> Duration {
        match (self.controller_start, self.controller_end) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        }
    }
}

impl Debug for FrameTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameTiming")
            .field("controller_time", &self.controller_time())
            .field("controller_start", &self.controller_start)
            .field("controller_end", &self.controller_end)
            .field("system_timings", &self.system_timings)
            .finish()
    }
}

#[derive(Clone)]
pub struct SystemTiming {
    pub label: &'static str,
    pub frame_init_start: Instant,
    pub frame_init_end: Instant,

    pub main_process_start: Instant,
    pub main_process_end: Instant,

    pub post_process_start: Instant,
    pub post_process_end: Instant,
}

impl Debug for SystemTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTiming")
            .field("label", &self.label)
            .field(
                "frame_init_time",
                &(self.frame_init_end - self.frame_init_start),
            )
            .field(
                "main_process_time",
                &(self.main_process_end - self.main_process_start),
            )
            .field(
                "post_process_time",
                &(self.post_process_end - self.post_process_start),
            )
            .field("frame_init_start", &self.frame_init_start)
            .field("frame_init_end", &self.frame_init_end)
            .field("frame_main_process_start", &self.main_process_start)
            .field("frame_main_process_end", &self.main_process_end)
            .field("frame_post_process_start", &self.post_process_start)
            .field("frame_post_process_end", &self.post_process_end)
            .finish()
    }
}

pub(in crate::engine) struct SystemProfiler {
    label: &'static str,
    frame_init_start: Option<Instant>,
    frame_init_end: Option<Instant>,
    main_process_start: Option<Instant>,
    main_process_end: Option<Instant>,
    post_process_start: Option<Instant>,
    post_process_end: Option<Instant>,
}

impl SystemProfiler {
    pub fn frame_init(&mut self, f: impl FnOnce()) {
        debug_assert!(self.frame_init_start.is_none() && self.frame_init_end.is_none());
        self.frame_init_start = Some(Instant::now());
        f();
        self.frame_init_end = Some(Instant::now());
    }

    pub fn main_process(&mut self, f: impl FnOnce()) {
        debug_assert!(self.main_process_start.is_none() && self.main_process_end.is_none());
        self.main_process_start = Some(Instant::now());
        f();
        self.main_process_end = Some(Instant::now());
    }

    pub fn post_process(&mut self, f: impl FnOnce()) {
        debug_assert!(self.post_process_start.is_none() && self.post_process_end.is_none());
        self.post_process_start = Some(Instant::now());
        f();
        self.post_process_end = Some(Instant::now());
    }

    pub fn reset(&mut self) -> SystemTiming {
        SystemTiming {
            label: self.label,
            frame_init_start: self.frame_init_start.take().unwrap(),
            frame_init_end: self.frame_init_end.take().unwrap(),
            main_process_start: self.main_process_start.take().unwrap(),
            main_process_end: self.main_process_end.take().unwrap(),
            post_process_start: self.post_process_start.take().unwrap(),
            post_process_end: self.post_process_end.take().unwrap(),
        }
    }
}

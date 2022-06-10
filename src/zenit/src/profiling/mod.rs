use std::time::{Duration, Instant};

pub mod frame_profiler;
pub use frame_profiler::*;

pub fn measure_time(f: impl FnOnce()) -> Duration {
    let start = Instant::now();
    f();
    let end = Instant::now();
    end.duration_since(start)
}

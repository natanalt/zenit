use std::time::{Duration, Instant};

pub mod frame_profiler;
pub use frame_profiler::*;

/// Measures the time it takes to execute given function.
/// Same accuracy limitations apply as with Rust's [`Instant`].
pub fn measure_time(f: impl FnOnce()) -> Duration {
    let start = Instant::now();
    f();
    let end = Instant::now();
    end.duration_since(start)
}

//! A forever rising, global counter.
//! Useful if you want to generate numbers from a source that will never repeat

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns next value from the counter.
/// 
/// ## Panics
/// Panics if an overflow is ever reached. With an unsigned 64-bit integer, it
/// can be considered unlikely.
/// 
/// If you were to generate 1 million new numbers every second, it'd take
/// over 584 942 years to get here.
pub fn next() -> u64 {
    let result = COUNTER.fetch_add(1, Ordering::SeqCst);
    if result == u64::MAX {
        panic!("How did we get here?");
    }
    result
}

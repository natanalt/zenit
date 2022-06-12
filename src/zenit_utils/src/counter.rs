use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A forever rising, global counter.
/// Useful if you want to generate numbers from a source that will never repeat
pub fn next() -> u64 {
    let result = COUNTER.fetch_add(1, Ordering::SeqCst);
    if result == u64::MAX {
        #[cold]
        fn overflow() {
            panic!("Global utility counter overflow!");
        }

        // If you were to generate 1 million new numbers every second, it'd take
        // over 584 942 years to get here.
        //
        // It's safe to assume that it's very unlikely for an overflow to be
        // reached.
        overflow()
    }
    result
}

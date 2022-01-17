//! Various utilities used by the various code

mod fnv1a;
pub use fnv1a::fnv1a_hash;

/// Useful where it's useful lol
/// 
/// Check existing usage for details
#[macro_export]
macro_rules! error_if {
    ($cond:expr, $reterr:expr) => {
        if $cond {
            return Err($reterr)?;
        }
    }
}

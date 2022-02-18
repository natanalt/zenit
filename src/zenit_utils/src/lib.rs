//! Various utilities used by the various code

pub mod fnv1a;
pub use fnv1a::fnv1a_hash;

pub mod packed;

pub type AnyResult<T = (), E = anyhow::Error> = anyhow::Result<T, E>;

/// Aligns the value. Alignment doesn't have to be a power of two.
/// 
/// ```
/// use zenit_utils::align;
/// assert_eq!(16, align(10, 8));
/// ```
pub fn align(n: u64, a: u64) -> u64 {
    (n + a - 1) / a * a
}

/// Used by the `ext_repr` proc macro
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum EnumParseError {
    #[error("invalid input")]
    InvalidInput
}

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

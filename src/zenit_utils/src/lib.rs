//! Various utilities used by the various code

pub mod fnv1a;
pub mod packed;

pub use fnv1a::fnv1a_hash;
pub use fnv1a::FnvHashExt;

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

/// Used by the [`zenit_proc::ext_repr`] proc macro
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum EnumParseError {
    #[error("invalid input")]
    InvalidInput,
}

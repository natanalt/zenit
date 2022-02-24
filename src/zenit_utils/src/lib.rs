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

/// Converts a 4-byte string into a u32
/// 
/// ## Example
/// ```
/// use zenit_utils::string_as_u32;
/// assert_eq!(string_as_u32("DXT3"), 0x33545844);
/// ```
pub const fn string_as_u32(s: &str) -> u32 {
    if s.len() == 4 {
        let bytes = s.as_bytes();
        let mut result = 0u32;
        result |= (bytes[0] as u32) << 0;
        result |= (bytes[1] as u32) << 8;
        result |= (bytes[2] as u32) << 16;
        result |= (bytes[3] as u32) << 24;
        result
    } else {
        panic!("Invalid string length");
    }
}

/// Used by the [`zenit_proc::ext_repr`] proc macro
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum EnumParseError {
    #[error("invalid input")]
    InvalidInput,
}

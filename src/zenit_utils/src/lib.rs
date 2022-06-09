//! Various utilities used by the various code

pub mod fnv1a;
pub mod packed;

pub use fnv1a::fnv1a_hash;
pub use fnv1a::FnvHashExt;

pub type AnyResult<T = (), E = anyhow::Error> = anyhow::Result<T, E>;

/// Shorthand for `Ok(())`, cause it looks ugly
pub fn ok<E>() -> Result<(), E> {
    Ok(())
}

/// Shorthand for `Default::default()` and such
pub fn default<T: Default>() -> T {
    T::default()
}

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

/// https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2
#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:expr) => {{
        #[repr(C)]
        pub struct AlignedAs<Align, Bytes: ?Sized> {
            pub _align: [Align; 0],
            pub bytes: Bytes,
        }

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

/// Includes a file into the program as a &'static slice of given type,
/// with guaranteed proper alignments.
#[macro_export]
macro_rules! include_bytes_as {
    ($target_ty:ty, $path:expr) => {{
        let size = ::std::mem::size_of::<$target_ty>();

        let bytes = $crate::include_bytes_align_as!($target_ty, $path);
        if bytes.len() % size != 0 {
            // Ideally this should be a compile error
            panic!("File doesn't evenly fit needed type");
        }

        let result: &'static [$target_ty] = unsafe {
            ::std::slice::from_raw_parts(bytes.as_ptr() as *const u32, bytes.len() / size)
        };

        result
    }};
}

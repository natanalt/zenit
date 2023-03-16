//! Various utilities used by the various code

// TODO: clean up zenit_utils

use byteorder::{NativeEndian, WriteBytesExt};
use std::io::Cursor;
use std::mem;

pub mod color;
pub mod counter;
pub mod math;
pub mod packed;

mod pool;
pub use pool::*;

mod arc_pool;
pub use arc_pool::*;

mod ascii_display;
pub use ascii_display::*;

pub mod fnv1a;
pub use fnv1a::fnv1a_hash;
pub use fnv1a::Fnv1aHashExt;

mod cell_ext;
pub use cell_ext::RefCellExt;

mod rwlock_ext;
pub use rwlock_ext::RwLockExt;

mod mutex_ext;
pub use mutex_ext::MutexExt;

mod thread_cell;
pub use thread_cell::ThreadCell;

pub type AnyResult<T = (), E = anyhow::Error> = anyhow::Result<T, E>;

/// Shorthand for `Ok(())`, cause it looks ugly
pub const fn ok<E>() -> Result<(), E> {
    Ok(())
}

/// Shorthand for `Default::default()` and such
pub fn default_fn<T: Default>() -> T {
    T::default()
}

/// Aligns the value. Alignment doesn't have to be a power of two.
///
/// ```
/// use zenit_utils::align;
/// assert_eq!(16, align(10, 8));
/// ```
pub const fn align(n: u64, a: u64) -> u64 {
    (n + a - 1) / a * a
}

/// Converts a 4-byte string into a u32 (little endian)
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

pub fn pack_floats(buffer: &[f32]) -> Vec<u8> {
    let mut result = Vec::with_capacity(buffer.len() * mem::size_of::<f32>());
    let mut cursor = Cursor::new(&mut result);

    for &value in buffer {
        cursor.write_f32::<NativeEndian>(value).unwrap();
    }

    result
}

/// Used by the [`zenit_proc::ext_repr`] proc macro
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum EnumParseError {
    #[error("invalid input")]
    InvalidInput,
}

/// Generates a match statement that verifies a discriminant value as a single "expression".
///
/// ## Example
/// ```
/// # use zenit_utils::discriminant_matches;
/// enum Example { Simple, Tupled(u32) }
///
/// let a = Example::Simple;
/// assert!(discriminant_matches!(a, Example::Simple));
///
/// // Tupled variants must match the parameters. Don't name them to not get warnings.
/// let b = Example::Tupled(123);
/// assert!(discriminant_matches!(b, Example::Tupled(_)));
/// ```
#[macro_export]
macro_rules! discriminant_matches {
    ($value:expr, $discriminant:pat) => {
        match $value {
            $discriminant => true,
            _ => false,
        }
    };
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

/// Includes a file into the program as a `&'static` slice of given type,
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

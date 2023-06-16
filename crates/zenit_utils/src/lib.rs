//! Various utilities used by the various code

// TODO: clean up zenit_utils, there's a TON of stuff that *used* to be used.

use byteorder::{NativeEndian, WriteBytesExt};
use std::io::Cursor;
use std::mem;

pub mod color;
pub mod counter;
pub mod math;
pub mod packed;

pub mod fnv1a;
pub use fnv1a::fnv1a_hash;
pub use fnv1a::Fnv1aHashExt;

mod pool;
pub use pool::*;

mod arc_pool;
pub use arc_pool::*;

mod ascii_display;
pub use ascii_display::*;

mod cell_ext;
pub use cell_ext::RefCellExt;

mod rwlock_ext;
pub use rwlock_ext::RwLockExt;

mod mutex_ext;
pub use mutex_ext::MutexExt;

mod thread_cell;
pub use thread_cell::ThreadCell;

mod result_ext;
pub use result_ext::*;

mod seekable_take;
pub use seekable_take::*;

pub type AnyResult<T = (), E = anyhow::Error> = Result<T, E>;

/// Shorthand for `Ok(())`, cause it looks ugly
pub const fn ok<E>() -> Result<(), E> {
    Ok(())
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
#[error("invalid input")]
pub struct EnumParseError;

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

/// Generator of `as_something` functions for enum implementations and containers of enums.
///
/// It's a bit weird in usage due to Rust's oddities, but you should grasp how it's used based on
/// existing usage (recommend using a code editor with a language server for listing these).
#[macro_export]
macro_rules! enum_getters {
    (
        $Type:ty, $value:expr, $self:ident,
        $(
            { $name:ident => $pattern:pat, $pattern_capture:expr => $Return:ty }
        )*
    ) => {
        impl $Type {
            $(
                pub fn $name(&$self) -> Option<&$Return> {
                    match $value {
                        $pattern => Some($pattern_capture),
                        _ => None,
                    }
                }
            )*
        }
    }
}


mod fnv1a;
pub use fnv1a::fnv1a_hash as hash;

mod primitive_reader;
pub use primitive_reader::PrimitiveReader as PrimitiveReader;

/// Return unwrapped `Option<T>` value, or returns retval if it's `None`
#[macro_export]
macro_rules! unwrap_or_return {
    ($opt:expr, $retval:expr) => {
        match $opt {
            Some(x) => x,
            None => return $retval,
        }
    }
}

/// Similar to `unwrap_or_return`
#[macro_export]
macro_rules! unwrap_or_return_err {
    ($opt:expr, $reterr:expr) => {
        match $opt {
            Some(x) => x,
            None => return Err($reterr)?,
        }
    }
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

#[macro_export]
macro_rules! unwrap_or_bail {
    ($opt:expr, $error:literal) => {
        match $opt {
            Some(x) => x,
            None => anyhow::bail!($error),
        }
    }
}

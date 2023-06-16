use crate::ok;
use std::fmt::{self, Display};

/// Wrapper type for displaying byte buffers that contain all, or mostly all ASCII text.
/// Any non-ASCII bytes are displayed as `\xNN` where `NN` is their hex code. `\` is reinterpreted
/// as `\\`. Its behavior is implemented through the [`Display`] trait.
///
/// ## Example
/// ```
/// # use zenit_utils::AsciiDisplay;
/// let a = AsciiDisplay(b"abc");
/// assert_eq!(a.to_string(), "abc");
///
/// let b = AsciiDisplay(b"a\xABbc\\");
/// assert_eq!(b.to_string(), "a\\xABbc\\\\");
/// ```
pub struct AsciiDisplay<'a>(pub &'a [u8]);

impl<'a> From<&'a [u8]> for AsciiDisplay<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> Display for AsciiDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0 {
            if byte == b'\\' {
                write!(f, "\\\\")?;
            } else if byte.is_ascii_graphic() {
                write!(f, "{}", byte as char)?;
            } else {
                write!(f, r"\x{byte:02X}")?;
            }
        }
        ok()
    }
}

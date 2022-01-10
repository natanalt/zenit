use byteorder::{ByteOrder, LE};
use std::fmt::{Debug, Display};

/// Represents a tree node's 4 byte identifier
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MungeName {
    /// Raw name bytes. Little endian, and can sometimes be interpreted as 4 ASCII characters.
    /// Use implementations of `From<u32>` and `Into<u32>` to get endian accurate conversions between
    /// `u32` and `MungeName`.
    pub raw: [u8; 4],
}

impl MungeName {
    /// Parses a 4-letter literal.
    /// 
    /// Example:
    /// ```
    /// MungeName::from_literal("scr_")
    /// ```
    /// 
    /// ## Panics
    /// Panics if the literal isn't 4 bytes (ASCII characters, I know...) long.
    /// 
    pub const fn from_literal(s: &str) -> MungeName {
        if s.len() != 4 {
            panic!("Invalid length");
        }

        let bytes = s.as_bytes();
        MungeName {
            raw: [
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
            ]
        }
    }
}

impl From<u32> for MungeName {
    fn from(value: u32) -> Self {
        let mut result = MungeName { raw: [0u8; 4] };
        LE::write_u32(&mut result.raw, value);
        result
    }
}

impl Into<u32> for MungeName {
    fn into(self) -> u32 {
        LE::read_u32(&self.raw)
    }
}

impl TryInto<String> for MungeName {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        fn is_accepted_name_char(c: char) -> bool {
            (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || (c == '_')
        }

        let mut result = String::with_capacity(4);
        for c in self.raw {
            if is_accepted_name_char(c as char) {
                result.push(c as char);
            } else {
                return Err(());
            }
        }
        Ok(result)
    }
}

impl Display for MungeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(name) = TryInto::<String>::try_into(self.clone()) {
            write!(f, "MungeName {{ \"{}\" }}", name)
        } else {
            write!(
                f,
                "MungeName {{ \\x{:02x}\\x{:02x}\\x{:02x}\\x{:02x} }}",
                self.raw[0], self.raw[1], self.raw[2], self.raw[3]
            )
        }
    }
}

impl Debug for MungeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

use std::fmt::{Debug, Display, Formatter};

pub mod reader;
pub mod writer;
pub mod parser;

/// Represents a node in a level file.
#[derive(Debug, Clone)]
pub struct LevelNode {
    pub name: NodeName,
    pub payload_offset: u64,
    pub payload_size: u32,
    pub payload: NodePayload,
}

#[derive(Debug, Clone)]
pub enum NodePayload {
    /// Represents raw data
    Raw(Vec<u8>),
    /// Represents all children whose metadata **was loaded**.
    Hierarchy(Vec<LevelNode>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeName(pub [u8; 4]);

impl NodeName {
    /// Creates a NodeName from a provided string slice. It's meant to be used in places like
    /// constant expressions.
    ///
    /// ## Panics
    /// Panics when the syntax is not trivially convertable into a `NodeName`.
    ///
    /// ## Examples
    /// Basic usage:
    /// ```
    /// use zenit_lvl::node::NodeName;
    /// 
    /// let name = NodeName::from_str("FMT_");
    /// assert_eq!(0x5f544d46u32, name.into());
    /// ```
    ///
    /// Invalid usage:
    /// ```should_panic
    /// use zenit_lvl::node::NodeName;
    /// 
    /// let _ = NodeName::from_str("too long");
    /// ```
    ///
    pub const fn from_str(s: &str) -> NodeName {
        if s.len() != 4 {
            panic!("Invalid node name length");
        }
        let bytes = s.as_bytes();
        NodeName([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    /// Returns a [`&str`] if this name can be interpreted as such.
    pub fn to_str(&self) -> Option<&str> {
        let is_valid = self.0.iter().all(|c| !u8::is_ascii_control(c));
        if is_valid {
            std::str::from_utf8(&self.0).ok()
        } else {
            None
        }
    }
}

impl Into<u32> for NodeName {
    fn into(self) -> u32 {
        u32::from_le_bytes(self.0)
    }
}

impl From<u32> for NodeName {
    fn from(value: u32) -> Self {
        NodeName(value.to_le_bytes())
    }
}

impl TryInto<String> for NodeName {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        #[rustfmt::skip]
        fn is_accepted_name_char(c: u8) -> bool {
            (c >= b'a' && c <= b'z') ||
            (c >= b'A' && c <= b'Z') ||
            (c >= b'0' && c <= b'9') ||
            (c == b'_')
        }

        self.0
            .into_iter()
            .map(|c| {
                if is_accepted_name_char(c) {
                    Ok(c as char)
                } else {
                    Err(())
                }
            })
            .collect()
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let inner = self.clone().try_into().unwrap_or_else(|_| {
            format!(
                "\\x{:02x} \\x{:02x} \\x{:02x} \\x{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            )
        });
        write!(f, "NodeName {{ {} }}", inner)
    }
}

impl Debug for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

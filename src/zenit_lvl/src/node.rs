use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct LevelNode {
    pub name: NodeName,
    pub offset: u64,
    pub size: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeName([u8; 4]);

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

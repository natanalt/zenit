use std::fmt::{self, Debug, Display, Formatter};

/// Convenience macro for creating [`NodeName`] instances using the [`NodeName::from_str`] function
/// You **must** have `NodeName` imported.
#[macro_export]
macro_rules! node {
    ($s:expr) => {
        NodeName::from_str($s)
    };
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
    /// use zenit_lvl_core::node::NodeName;
    ///
    /// let name = NodeName::from_str("FMT_");
    /// assert_eq!(0x5f544d46u32, name.into());
    /// ```
    ///
    /// Invalid usage:
    /// ```should_panic
    /// use zenit_lvl_core::node::NodeName;
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

    /// The most common node name of root nodes, `ucfb`
    pub const fn root() -> NodeName {
        NodeName::from_str("ucfb")
    }

    /// Creates a `NodeName` out of an FNV-1a hash of given string
    pub fn from_hash(s: &str) -> NodeName {
        zenit_utils::fnv1a_hash(s.as_bytes()).into()
    }

    /// Returns a [`&str`] if this name can be interpreted as such.
    pub fn as_str(&self) -> Option<&str> {
        self.0
            .into_iter()
            .all(is_accepted_name_byte)
            .then(|| std::str::from_utf8(&self.0).ok())
            .flatten()
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
        self.0
            .into_iter()
            .map(|c| {
                if is_accepted_name_byte(c) {
                    Ok(c as char)
                } else {
                    Err(())
                }
            })
            .collect()
    }
}

impl TryFrom<&str> for NodeName {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        if bytes.len() == 4 {
            Ok(Self([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Err(())
        }
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Ok(s) = TryInto::<String>::try_into(self.clone()) {
            write!(f, "{}", s)
        } else {
            write!(f, "{:?}", &self.0)
        }
    }
}

impl Debug for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("NodeName");

        if let Ok(s) = TryInto::<String>::try_into(self.clone()) {
            tuple.field(&s);
        } else {
            tuple.field(&self.0);
        }

        tuple.finish()
    }
}

#[rustfmt::skip]
pub fn is_accepted_name_byte(c: u8) -> bool {
    (c >= b'a' && c <= b'z') ||
    (c >= b'A' && c <= b'Z') ||
    (c >= b'0' && c <= b'9') ||
    (c == b'_')
}

//! Reader of munged .lvl files
//! Format documentation is available here:
//! https://gist.github.com/natanalt/2ef697e53e56d6abfb42a644f6317d68

use crate::{unwrap_or_return, utils::PrimitiveReader};
use byteorder::{ByteOrder, LE};
use std::fmt::{Debug, Display};

/// Represents a node in a munged level tree
#[derive(Clone)]
pub struct TreeNode<'s> {
    name: NodeName,
    data: &'s [u8],
}

impl<'s> TreeNode<'s> {
    /// Attempts to parse given block as node data.
    ///
    /// This function will fail if:
    ///  * the node's size is smaller than 8 bytes
    ///  * the node's name begins with a nul byte
    ///  * the node's size is larger than size of provided `data` slice
    pub fn parse(data: &'s [u8]) -> Option<Self> {
        if data.len() < 8 || data[0] == 0 {
            return None;
        }

        let name = NodeName::from(LE::read_u32(&data[0..4]));
        let size = LE::read_u32(&data[4..8]);

        if size as usize + 8 > data.len() {
            return None;
        }

        Some(Self {
            name,
            data: &data[8..(size + 8) as usize],
        })
    }

    /// Attempts to parse this node's children - if it has any. If the reading process fails (most likely because the
    /// node doesn't contain a valid hierarchy), `None` is returned.
    pub fn parse_children(&self) -> Option<Vec<TreeNode<'s>>> {
        let first_attempt =
            Self::parse_children_inner(PrimitiveReader::new(self.data), None);
        if first_attempt.is_some() {
            first_attempt
        } else {
            // Try again, assuming that the first 4 bytes specify the node count
            let mut parser = PrimitiveReader::new(self.data);
            let count = unwrap_or_return!(parser.read_u32(), None);
            Self::parse_children_inner(parser, Some(count))
        }
    }

    /// Internal parsing function
    fn parse_children_inner(
        mut parser: PrimitiveReader<'s, LE>,
        node_limit: Option<u32>,
    ) -> Option<Vec<TreeNode<'s>>> {
        let node_limit = node_limit.or(Some(u32::MAX)).unwrap();
        let mut parsed_so_far = 0;
        let mut result = Vec::new();

        while parser.remaining_bytes() > 0 {
            parser.skip_specific_bytes(0);
            if parser.remaining_bytes() == 0 {
                break;
            }

            if parsed_so_far == node_limit {
                break;
            }

            let name = NodeName::from(unwrap_or_return!(parser.read_u32(), None));
            let size = unwrap_or_return!(parser.read_u32(), None);
            if size as usize > parser.remaining_bytes() {
                println!("Parsing failed: invalid size {:#?}: {:x}", name, size);
                return None;
            }

            let data = parser.data();
            let offset = parser.offset();
            result.push(TreeNode {
                name,
                data: &data[offset..offset + size as usize],
            });
            parsed_so_far += 1;
            parser.skip_bytes(size as usize);
        }

        Some(result)
    }

    /// Returns the node's name
    pub fn name(&self) -> NodeName {
        self.name
    }

    /// Returns a reference to this node's data
    pub fn data(&self) -> &'s [u8] {
        self.data
    }
}

impl<'s> Display for TreeNode<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TreeNode {{ {} ({} byte{}) }}",
            self.name,
            self.data.len(),
            if self.data.len() == 1 { "" } else { "s" }
        )
    }
}

impl<'s> Debug for TreeNode<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// Represents a tree node's 4 byte identifier
#[derive(Clone, Copy)]
pub struct NodeName {
    /// Raw name bytes. Little endian, and can sometimes be interpreted as 4 ASCII characters.
    /// Use implementations of `From<u32>` and `Into<u32>` to get endian accurate conversions between
    /// `u32` and `NodeName`.
    pub raw: [u8; 4],
}

impl From<u32> for NodeName {
    fn from(value: u32) -> Self {
        let mut result = NodeName { raw: [0u8; 4] };
        LE::write_u32(&mut result.raw, value);
        result
    }
}

impl Into<u32> for NodeName {
    fn into(self) -> u32 {
        LE::read_u32(&self.raw)
    }
}

impl TryInto<String> for NodeName {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
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

fn is_accepted_name_char(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || (c == '_')
}

impl Display for NodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(name) = TryInto::<String>::try_into(self.clone()) {
            write!(f, "NodeName {{ \"{}\" }}", name)
        } else {
            write!(
                f,
                "NodeName {{ \\x{:02x}\\x{:02x}\\x{:02x}\\x{:02x} }}",
                self.raw[0], self.raw[1], self.raw[2], self.raw[3]
            )
        }
    }
}

impl Debug for NodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

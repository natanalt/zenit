//! Reader of munged .lvl files
//! 
//! Format documentation is available here:
//! https://gist.github.com/natanalt/2ef697e53e56d6abfb42a644f6317d68
//! 

use crate::{unwrap_or_return_err, utils::PrimitiveReader, AnyResult};
use byteorder::{ByteOrder, LE};
use log::error;
use std::fmt::{Debug, Display};
use thiserror::Error;

/// Represents a node in a munged level tree
#[derive(Clone)]
pub struct TreeNode<'s> {
    name: NodeName,
    data: &'s [u8],
}

#[derive(Debug, Error, Clone, Copy)]
pub enum BasicParseError {
    #[error("Data size below 8 bytes")]
    TooSmall,
    #[error("Data size doesn't match node's size")]
    SizeMismatch,
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ParseChildrenError {
    #[error("Data doesn't seem to contain child nodes")]
    IncorrectFormat,
}

pub fn parse<'s>(data: &'s [u8]) -> AnyResult<TreeNode<'s>> {
    Ok(TreeNode::parse(data)?)
}

impl<'s> TreeNode<'s> {
    /// Attempts to parse given block as node data.
    ///
    /// This function will fail if:
    ///  * the node's size is smaller than 8 bytes
    ///  * the node's name begins with a nul byte
    ///  * the node's size is larger than size of provided `data` slice
    pub fn parse(data: &'s [u8]) -> Result<Self, BasicParseError> {
        if data.len() < 8 || data[0] == 0 {
            return Err(BasicParseError::TooSmall);
        }

        let name = NodeName::from(LE::read_u32(&data[0..4]));
        let size = LE::read_u32(&data[4..8]);

        if size as usize + 8 > data.len() {
            return Err(BasicParseError::SizeMismatch);
        }

        Ok(Self {
            name,
            data: &data[8..(size + 8) as usize],
        })
    }

    /// Attempts to parse this node's children - if it has any. If the reading process fails (most likely because the
    /// node doesn't contain a valid hierarchy), `None` is returned.
    pub fn parse_children(&self) -> Result<Vec<TreeNode<'s>>, ParseChildrenError> {
        let first_attempt = Self::parse_children_inner(PrimitiveReader::new(self.data), None);
        if first_attempt.is_ok() {
            first_attempt
        } else {
            // Try again, assuming that the first 4 bytes specify the node count
            let mut parser = PrimitiveReader::new(self.data);
            let count = parser
                .read_u32()
                .ok_or(ParseChildrenError::IncorrectFormat)?;
            Self::parse_children_inner(parser, Some(count))
        }
    }

    /// Internal parsing function
    fn parse_children_inner(
        mut parser: PrimitiveReader<'s, LE>,
        node_limit: Option<u32>,
    ) -> Result<Vec<TreeNode<'s>>, ParseChildrenError> {
        let node_limit = node_limit.unwrap_or(u32::MAX);
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

            let name = NodeName::from(unwrap_or_return_err!(
                parser.read_u32(),
                ParseChildrenError::IncorrectFormat
            ));
            let size = parser
                .read_u32()
                .ok_or(ParseChildrenError::IncorrectFormat)?;
            if size as usize > parser.remaining_bytes() {
                error!("Parsing failed: invalid size {:#?}: {:x}", name, size);
                return Err(ParseChildrenError::IncorrectFormat);
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

        Ok(result)
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NodeName {
    /// Raw name bytes. Little endian, and can sometimes be interpreted as 4 ASCII characters.
    /// Use implementations of `From<u32>` and `Into<u32>` to get endian accurate conversions between
    /// `u32` and `NodeName`.
    pub raw: [u8; 4],
}

impl NodeName {
    /// Parses a 4-letter literal.
    /// 
    /// Example:
    /// ```
    /// NodeName::from_literal("scr_")
    /// ```
    /// 
    /// ## Panics
    /// Panics if the literal isn't 4 bytes (ASCII characters, I know...) long.
    /// 
    pub const fn from_literal(s: &str) -> NodeName {
        if s.len() != 4 {
            panic!("Invalid length");
        }

        let bytes = s.as_bytes();
        NodeName {
            raw: [
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
            ]
        }
    }
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

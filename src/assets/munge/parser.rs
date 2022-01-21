use crate::AnyResult;

use super::{MungeName, MungeNode, MungeTreeNode};
use byteorder::{ReadBytesExt, LE, WriteBytesExt};
use std::{
    ffi::CStr,
    io::{self, Cursor, Read, Seek, SeekFrom},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("This data doesn't contain valid node data")]
    InvalidNode,
    #[error("IO error has occurred: {0}")]
    IoError(#[from] io::Error),
}

impl MungeNode {
    /// Parses a node. It only resolves the name and length, and doesn't attempt to parse children
    /// nodes, see `parse_children` for that.
    pub fn parse<Reader: Read + Seek>(r: &mut Reader) -> Result<MungeNode, ParseError> {
        let name = MungeName::from(r.read_u32::<LE>()?);
        if name.raw[0] == 0 {
            return Err(ParseError::InvalidNode);
        }

        let length = r.read_u32::<LE>()?;
        let offset = r.stream_position()?;

        let max_length = {
            let old_pos = r.stream_position()?;
            let len = r.seek(SeekFrom::End(0))?;
            r.seek(SeekFrom::Start(old_pos))?;
            len - offset
        };

        if length as u64 > max_length {
            return Err(ParseError::InvalidNode);
        }

        Ok(MungeNode {
            name,
            offset,
            length,
        })
    }

    pub fn read_with_header<Reader: Read + Seek>(&self, r: &mut Reader) -> io::Result<Vec<u8>> {
        let mut result = Vec::new();
        result.write_u32::<LE>(self.name.into())?;
        result.write_u32::<LE>(self.length)?;
        result.append(&mut self.read_contents(r)?);
        Ok(result)
    }

    /// Reads the internal node data into a `Vec<u8>` from given `Read + Seek` reader.
    pub fn read_contents<Reader: Read + Seek>(&self, r: &mut Reader) -> io::Result<Vec<u8>> {
        let length = self.length as usize;
        let mut result = Vec::with_capacity(length);
        result.resize(length, 0);

        r.seek(SeekFrom::Start(self.offset))?;
        r.read(&mut result)?;

        Ok(result)
    }

    /// Attempts to parse the node contents as a null-terminated UTF-8 String
    pub fn read_string<Reader: Read + Seek>(&self, r: &mut Reader) -> AnyResult<String> {
        Ok(CStr::from_bytes_with_nul(&self.read_contents(r)?)?
            .to_str()?
            .to_owned())
    }

    pub fn read_into_cursor<Reader: Read + Seek>(
        &self,
        r: &mut Reader,
    ) -> io::Result<Cursor<Vec<u8>>> {
        Ok(Cursor::new(self.read_contents(r)?))
    }

    /// Attempts to interpret the internal node data as if it contained child nodes. This is not
    /// a recursive operation. `BufReader` is recommended for this.
    pub fn parse_children<Reader: Read + Seek>(
        &self,
        r: &mut Reader,
    ) -> io::Result<Option<Vec<MungeNode>>> {
        if self.length < 8 || self.length == u32::MAX {
            return Ok(None);
        }

        // Attempt to parse:
        //  1. without any offsets
        //  2. with a single 4 byte offset (sometimes nodes encode their child counts there)
        // Note: IO error is an instant failure
        r.seek(SeekFrom::Start(self.offset))?;
        Ok(match parse_children_inner(r, self.length)? {
            Some(x) => Some(x),
            None => {
                r.seek(SeekFrom::Start(self.offset + 4))?;
                parse_children_inner(r, self.length - 4)?
            }
        })
    }

    pub fn parse_tree<Reader: Read + Seek>(
        &self,
        r: &mut Reader,
        max_depth: Option<u32>,
    ) -> io::Result<MungeTreeNode> {
        let max_depth = max_depth.unwrap_or(u32::MAX);

        if max_depth == 0 {
            return Ok(MungeTreeNode {
                node: self.clone(),
                children: Vec::new(),
            });
        }

        let children = {
            let base = self.parse_children(r)?;
            if let Some(base) = base {
                base
            } else {
                return Ok(MungeTreeNode {
                    node: self.clone(),
                    children: Vec::new(),
                });
            }
        };

        Ok(MungeTreeNode {
            node: self.clone(),
            children: {
                let mut result = Vec::new();
                for child in children {
                    result.push(child.parse_tree(r, Some(max_depth - 1))?);
                }
                result
            },
        })
    }
}

impl MungeTreeNode {
    /// Parses a new node, and its children
    pub fn parse<Reader: Read + Seek>(
        r: &mut Reader,
        max_depth: Option<u32>,
    ) -> Result<Self, ParseError> {
        Ok(MungeNode::parse(r)?.parse_tree(r, max_depth)?)
    }

    /// Parses children of this node entry and puts them into self children vector. Goes
    /// recursively, if possible, up to `max_depth` (if None, then there's no limit)
    pub fn parse_tree<Reader: Read + Seek>(
        &mut self,
        r: &mut Reader,
        max_depth: Option<u32>,
    ) -> io::Result<()> {
        *self = self.node.parse_tree(r, max_depth)?;
        Ok(())
    }
}

fn parse_children_inner<Reader: Read + Seek>(
    r: &mut Reader,
    size_limit: u32,
) -> io::Result<Option<Vec<MungeNode>>> {
    let mut result = Vec::new();
    let mut parsed_bytes = 0;

    let size_limit = size_limit as u64;

    let skip_zeroes = |r: &mut Reader, parsed_bytes: &mut u64| {
        let mut zeroes_found = false;
        while *parsed_bytes < size_limit {
            let next = r.read_u8()?;
            if next != 0 {
                r.seek(SeekFrom::Current(-1))?;
                break;
            }
            *parsed_bytes += 1;
            zeroes_found = true;
        }
        io::Result::<bool>::Ok(zeroes_found)
    };

    while parsed_bytes < size_limit {
        if skip_zeroes(r, &mut parsed_bytes)? {
            continue;
        }

        // Non-zero data that can't be a valid node
        if size_limit - parsed_bytes < 8 {
            return Ok(None);
        }

        let child = match MungeNode::parse(r) {
            Ok(x) => x,
            Err(err) => match err {
                ParseError::InvalidNode => return Ok(None),
                ParseError::IoError(err) => return Err(err),
            },
        };

        let taken_bytes = (child.length as u64) + 8;

        if parsed_bytes + taken_bytes > size_limit {
            return Ok(None);
        }

        parsed_bytes += taken_bytes;
        r.seek(SeekFrom::Current(taken_bytes as i64 - 8))?;

        result.push(child);
    }

    Ok(Some(result))
}

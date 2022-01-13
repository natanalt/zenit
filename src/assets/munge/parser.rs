use super::{MungeName, MungeNode};
use byteorder::{ReadBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("This data doesn't contain valid node data")]
    InvalidNode,
    #[error("IO error has occurred: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ParseChildrenError {
    #[error("This node doesn't contain valid children")]
    NoChildren,
    #[error("IO error has occurred: {0}")]
    IoError(#[from] io::Error),
}

impl MungeNode {
    /// Parses a node. It only resolves the name and length, and doesn't attempt to parse children
    /// nodes, see `parse_children` for that.
    pub fn parse<Reader: Read + Seek>(mut r: Reader) -> Result<MungeNode, ParseError> {
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

    /// Attempts to interpret the internal node data as if it contained child nodes. This is not
    /// a recursive operation. `BufReader` is recommended for this.
    pub fn parse_children<Reader: Read + Seek>(
        &self,
        mut r: Reader,
    ) -> Result<Vec<MungeNode>, ParseChildrenError> {
        if self.length < 8 || self.length == u32::MAX {
            return Err(ParseChildrenError::NoChildren);
        }

        // Attempt to parse:
        //  1. without any offsets
        //  2. with a single 4 byte offset (sometimes nodes encode their child counts there)
        // Note: IO error is an instant failure
        r.seek(SeekFrom::Start(self.offset))?;
        parse_children_inner(&mut r, self.length).or_else(|err| match err {
            ParseChildrenError::NoChildren => {
                r.seek(SeekFrom::Start(self.offset + 4))?;
                parse_children_inner(&mut r, self.length - 4)
            }
            ParseChildrenError::IoError(_) => Err(err),
        })
    }
}

fn parse_children_inner<Reader: Read + Seek>(
    mut r: Reader,
    size_limit: u32,
) -> Result<Vec<MungeNode>, ParseChildrenError> {
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
        if skip_zeroes(&mut r, &mut parsed_bytes)? {
            continue;
        }

        // Non-zero data that can't be a valid node
        if size_limit - parsed_bytes < 8 {
            return Err(ParseChildrenError::NoChildren);
        }

        let child = MungeNode::parse(&mut r).map_err(|err| match err {
            ParseError::InvalidNode => ParseChildrenError::NoChildren,
            ParseError::IoError(e) => ParseChildrenError::IoError(e),
        })?;
        let taken_bytes = (child.length as u64) + 8;

        if parsed_bytes + taken_bytes > size_limit {
            return Err(ParseChildrenError::NoChildren);
        }

        parsed_bytes += taken_bytes;
        r.seek(SeekFrom::Current(taken_bytes as i64 - 8))?;

        result.push(child);
    }

    Ok(result)
}

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
    pub fn parse<Reader: Read + Seek>(r: &Reader) -> Result<MungeNode, ParseError> {
        let name = MungeName::from(r.read_u32::<LE>()?);
        if name.raw[0] == 0 {
            return Err(ParseError::InvalidNode);
        }

        let length = r.read_u32::<LE>()?;
        let offset = r.stream_position()?;
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
        r: &Reader,
    ) -> Result<Vec<MungeNode>, ParseChildrenError> {
        if self.length < 8 || self.length == u32::MAX {
            return Err(ParseChildrenError::NoChildren);
        }

        // Attempt to parse:
        //  1. without any offsets
        //  2. with a single 4 byte offset (sometimes nodes encode their child counts there)
        // Note: IO error is an instant failure
        r.seek(SeekFrom::Start(self.offset))?;
        parse_children_inner(r, self.length).or_else(|err| match err {
            ParseChildrenError::NoChildren => {
                r.seek(SeekFrom::Start(self.offset + 4))?;
                parse_children_inner(r, self.length - 4)
            }
            ParseChildrenError::IoError(_) => Err(err),
        })
    }
}

fn parse_children_inner<Reader: Read + Seek>(
    r: &Reader,
    size_limit: u32,
) -> Result<Vec<MungeNode>, ParseChildrenError> {
    let mut result = Vec::new();
    let mut parsed_bytes = 0;

    let mut skip_zeroes = || {
        let mut zeroes_found = false;
        while parsed_bytes < size_limit {
            let next = r.read_u8()?;
            if next != 0 {
                r.seek(SeekFrom::Current(-1))?;
                break;
            }
            parsed_bytes += 1;
            zeroes_found = true;
        }
        io::Result::<bool>::Ok(zeroes_found)
    };

    while parsed_bytes < size_limit {
        if skip_zeroes()? {
            continue;
        }

        let child = MungeNode::parse(r).map_err(|_| ParseChildrenError::NoChildren)?;
        let taken_bytes = child.length + 8;
        if taken_bytes + parsed_bytes > size_limit {
            return Err(ParseChildrenError::NoChildren);
        }
        result.push(child);
    }

    Ok(result)
}

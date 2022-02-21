use super::{LevelNode, NodeName};
use byteorder::{ReadBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom};
use thiserror::Error;

// TODO: add a basic parsing test file and parse it

pub trait NodeReadExt
where
    Self: Sized,
{
    /// Parses a node. It only resolves the name and length, and doesn't attempt to parse children
    /// nodes, see `read_children` for that.
    fn parse_header<Reader: Read + Seek>(r: &mut Reader) -> Result<Self, ReadError>;

    /// Reads the raw payload data from the node. Doesn't include the header.
    fn parse_raw_data<Reader: Read + Seek>(&mut self, r: &mut Reader) -> io::Result<Vec<u8>>;

    /// Interprets this node as a parent node and returns a list of children, or None if no hierarchy
    /// is recognized.
    fn parse_children<Reader: Read + Seek>(
        &self,
        r: &mut Reader,
    ) -> io::Result<Option<Vec<LevelNode>>>;
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("This data doesn't contain valid node data")]
    InvalidNode,
    #[error("IO error has occurred: {0}")]
    IoError(#[from] io::Error),
}

impl NodeReadExt for LevelNode {
    fn parse_header<Reader: Read + Seek>(r: &mut Reader) -> Result<Self, ReadError> {
        let name = NodeName::from(r.read_u32::<LE>()?);
        if name.0[0] == 0 {
            return Err(ReadError::InvalidNode);
        }

        let payload_size = r.read_u32::<LE>()?;
        let payload_offset = r.stream_position()?;

        let remaining_length = {
            let old_pos = r.stream_position()?;
            let len = r.seek(SeekFrom::End(0))?;
            r.seek(SeekFrom::Start(old_pos))?;
            len - payload_offset
        };

        if payload_size as u64 > remaining_length {
            return Err(ReadError::InvalidNode);
        }

        Ok(Self {
            name,
            payload_offset,
            payload_size,
        })
    }

    fn parse_raw_data<R: Read + Seek>(&mut self, r: &mut R) -> io::Result<Vec<u8>> {
        r.seek(SeekFrom::Start(self.payload_offset))?;
        let mut buffer = vec![0; self.payload_size as usize];
        r.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn parse_children<Reader: Read + Seek>(
        &self,
        r: &mut Reader,
    ) -> io::Result<Option<Vec<LevelNode>>> {
        if self.payload_size < 8 || self.payload_size == u32::MAX {
            return Ok(None);
        }

        // Attempt to parse:
        //  1. without any offsets
        //  2. with a single 4 byte offset (sometimes nodes encode their child counts there)
        r.seek(SeekFrom::Start(self.payload_offset))?;
        Ok(read_children_list_inner(r, self.payload_size)?.map_or_else(
            || {
                r.seek(SeekFrom::Start(self.payload_offset + 4))?;
                io::Result::Ok(read_children_list_inner(r, self.payload_size - 4)?)
            },
            |v| Ok(Some(v)),
        )?)
    }
}

fn read_children_list_inner<Reader: Read + Seek>(
    r: &mut Reader,
    size_limit: u32,
) -> io::Result<Option<Vec<LevelNode>>> {
    let mut r = r.take(size_limit as u64);
    let mut result = Vec::new();

    while r.with_skipped_padding()? {
        let name = r.read_u32::<LE>()?;
        let payload_size = r.read_u32::<LE>()?;
        let payload_offset = r.get_mut().stream_position()?;
        r.skip(payload_size as u64)?;
        result.push(LevelNode {
            name: NodeName::from(name),
            payload_offset,
            payload_size,
        });
    }

    Ok(Some(result))
}

trait SeekTakeExt {
    /// Skips padding (zeroes), and reports whether there's more data to be read
    /// after the skip.
    fn with_skipped_padding(&mut self) -> io::Result<bool>;
    /// Skips a set amount of bytes, returns UnexpectedEof if not possible
    fn skip(&mut self, bytes: u64) -> io::Result<()>;
}

impl<T: Read + Seek> SeekTakeExt for io::Take<T> {
    fn with_skipped_padding(&mut self) -> io::Result<bool> {
        loop {
            if self.limit() == 0 {
                break Ok(false);
            }

            if self.read_u8()? != 0 {
                self.set_limit(self.limit() + 1);
                self.get_mut().seek(SeekFrom::Current(-1))?;
                break Ok(true);
            }
        }
    }

    fn skip(&mut self, bytes: u64) -> io::Result<()> {
        if self.limit() <= bytes {
            self.get_mut().seek(SeekFrom::Current(bytes as i64))?;
            Ok(())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }
}

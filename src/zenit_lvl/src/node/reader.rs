use super::{LevelNode, NodeName, NodePayload};
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
    fn read<Reader: Read + Seek>(r: &mut Reader) -> Result<Self, ReadError>;

    /// Reads the raw payload data from the node. Doesn't include the header
    fn read_raw_data<R: Read + Seek>(&mut self, r: &mut R) -> io::Result<()>;

    /// Reads a tree of children based on a filter closure and inserts it into local children list.
    ///
    /// /// Example:
    /// ```no_run
    /// use zenit_lvl::node::{LevelNode, NodeName, read::{NodeReadExt, accept_children_of}};
    /// use std::fs::File;
    ///
    /// let mut file: File = todo!("example source, in this case, an open file");
    /// let mut node: LevelNode = todo!("your node goes here");
    ///
    /// // Only reads full hierarchies of script, texture, world and sound nodes
    /// let read_node_count = node.read_children(&mut file, accept_children_of(
    ///     1, // depth 1 -> children of file root
    ///     &[
    ///         NodeName::from_str("scr_"),
    ///         NodeName::from_str("tex_"),
    ///         NodeName::from_str("wrld"),
    ///         NodeName::from_str("snd_"),
    ///     ],
    /// ));
    /// ```
    fn read_children<R: Read + Seek, FilterFn: Fn(u32, NodeName) -> ReadDecision>(
        &mut self,
        r: &mut R,
        f: FilterFn,
    ) -> io::Result<usize>;

    /// Interprets this node as a parent node and returns a list of children, or None if no hierarchy
    /// is recognized.
    fn read_children_list<Reader: Read + Seek>(
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

#[derive(Debug, Clone, Copy)]
pub enum ReadDecision {
    /// The node should be skipped
    Ignore,
    /// The node should be included in the child list with its payload ignored
    Include,
    /// The node should be included in the child list, and its payload should be interpreted as
    /// hierarchy, or skipped if there's none
    IncludeAndParse,
}

impl NodeReadExt for LevelNode {
    fn read<Reader: Read + Seek>(r: &mut Reader) -> Result<Self, ReadError> {
        let name = NodeName::from(r.read_u32::<LE>()?);
        if name.0[0] == 0 {
            return Err(ReadError::InvalidNode);
        }

        let payload_size = r.read_u32::<LE>()?;
        let payload_offset = r.stream_position()?;

        let max_length = {
            let old_pos = r.stream_position()?;
            let len = r.seek(SeekFrom::End(0))?;
            r.seek(SeekFrom::Start(old_pos))?;
            len - payload_offset
        };

        if payload_size as u64 > max_length {
            return Err(ReadError::InvalidNode);
        }

        Ok(Self {
            name,
            payload_offset,
            payload_size,
            payload: NodePayload::Raw(vec![]),
        })
    }

    fn read_raw_data<R: Read + Seek>(&mut self, r: &mut R) -> io::Result<()> {
        r.seek(SeekFrom::Start(self.payload_offset))?;
        let mut buffer = vec![0; self.payload_size as usize];
        r.read_exact(&mut buffer)?;
        self.payload = NodePayload::Raw(buffer);
        Ok(())
    }

    fn read_children<R: Read + Seek, FilterFn: Fn(u32, NodeName) -> ReadDecision>(
        &mut self,
        r: &mut R,
        f: FilterFn,
    ) -> io::Result<usize> {
        Ok(read_children_inner(self, r, &f, 0)?)
    }

    fn read_children_list<Reader: Read + Seek>(
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

fn read_children_inner<R: Read + Seek, FilterFn: Fn(u32, NodeName) -> ReadDecision>(
    base: &mut LevelNode,
    r: &mut R,
    f: &FilterFn,
    depth: u32,
) -> io::Result<usize> {
    match base.read_children_list(r)? {
        Some(children) => Ok({
            let mut total = 0;

            let base_children = match &mut base.payload {
                NodePayload::Raw(_) => panic!("shouldn't happen"),
                NodePayload::Hierarchy(b) => b,
            };

            for mut child in children {
                match f(depth, child.name) {
                    ReadDecision::Ignore => {}
                    ReadDecision::Include => {
                        base_children.push(child);
                        total += 1;
                    }
                    ReadDecision::IncludeAndParse => {
                        read_children_inner(&mut child, r, f, depth + 1)?;
                        base_children.push(child);
                        total += 1;
                    }
                }
            }

            total
        }),
        None => Ok(0),
    }
}

fn read_children_list_inner<Reader: Read + Seek>(
    r: &mut Reader,
    size_limit: u32,
) -> io::Result<Option<Vec<LevelNode>>> {
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

        let child = match LevelNode::read(r) {
            Ok(x) => x,
            Err(err) => match err {
                ReadError::InvalidNode => return Ok(None),
                ReadError::IoError(err) => return Err(err),
            },
        };

        let taken_bytes = (child.payload_size as u64) + 8;

        if parsed_bytes + taken_bytes > size_limit {
            return Ok(None);
        }

        parsed_bytes += taken_bytes;
        r.seek(SeekFrom::Current(taken_bytes as i64 - 8))?;

        result.push(child);
    }

    Ok(Some(result))
}

/// Template for `LevelNode::read_children`'s FilterFn. Continually returns IncludeAndTryParsing.
pub fn accept_all(_depth: u32, _name: NodeName) -> ReadDecision {
    ReadDecision::IncludeAndParse
}

pub fn accept_until_depth<const MD: u32>(depth: u32, _name: NodeName) -> ReadDecision {
    if depth <= MD {
        ReadDecision::IncludeAndParse
    } else {
        ReadDecision::Ignore
    }
}

/// Template for level node children reading, letting you only parse hierarchies of base nodes
/// you accept.
///
/// Someday this may be replaced with const generics once they let passing more complex objects,
/// like, in this case, arrays.
pub fn accept_children_of<'a>(
    starting_depth: u32,
    accepted: &'a [NodeName],
) -> impl Fn(u32, NodeName) -> ReadDecision + 'a {
    move |depth, name| {
        assert!(depth >= starting_depth);
        if depth == starting_depth {
            if accepted.contains(&name) {
                ReadDecision::IncludeAndParse
            } else {
                ReadDecision::Ignore
            }
        } else {
            ReadDecision::IncludeAndParse
        }
    }
}

pub fn accept_children_of_str(
    starting_depth: u32,
    accepted_str: &[&str],
) -> impl Fn(u32, NodeName) -> ReadDecision {
    let accepted = accepted_str
        .into_iter()
        .map(|s| NodeName::from_str(s))
        .collect::<Vec<NodeName>>();

    move |depth, name| {
        assert!(depth >= starting_depth);
        if depth == starting_depth {
            if accepted.contains(&name) {
                ReadDecision::IncludeAndParse
            } else {
                ReadDecision::Ignore
            }
        } else {
            ReadDecision::IncludeAndParse
        }
    }
}

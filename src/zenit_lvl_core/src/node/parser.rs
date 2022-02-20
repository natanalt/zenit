use zenit_utils::{packed::PackedParser, AnyResult};

use crate::node::LevelNode;
use std::io::{Read, Seek, SeekFrom};

pub trait NodeParser
where
    Self: Sized,
{
    /// Parses `Self` from given node, using provided reader. Initial seek position is undefined.
    fn parse_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self>;
}

impl<T: PackedParser> NodeParser for T {
    fn parse_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        r.seek(SeekFrom::Start(raw.payload_offset))?;
        Ok(Self::parse_packed(&mut r.take(raw.payload_size as u64))?)
    }
}

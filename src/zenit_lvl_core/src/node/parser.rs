use super::LevelNode;
use std::{io::{Read, Seek, SeekFrom}, marker::PhantomData};
use zenit_utils::{packed::PackedParser, AnyResult};

pub trait NodeParser
where
    Self: Sized,
{
    /// Parses `Self` from given node, using provided reader. Initial seek position is undefined.
    fn parse_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self>;
}

impl NodeParser for LevelNode {
    fn parse_node<R: Read + Seek>(raw: LevelNode, _: &mut R) -> AnyResult<Self> {
        Ok(raw)
    }
}

impl<T: PackedParser> NodeParser for T {
    fn parse_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        r.seek(SeekFrom::Start(raw.payload_offset))?;
        Ok(Self::parse_packed(&mut r.take(raw.payload_size as u64))?)
    }
}

/// Represents data that is not meant to be immediately loaded from its node. Instead, its
/// metadata is cached, and available for reading on demand later on. Useful when loading
/// large nodes whose binary payloads are meant to be selectively loaded and/or ignored.
#[derive(Debug, Clone)]
pub struct LazyData<T: NodeParser> {
    pub node: LevelNode,
    phantom: PhantomData<T>,
}

impl<T: NodeParser> LazyData<T> {
    pub fn new(node: LevelNode) -> Self {
        Self {
            node,
            phantom: Default::default(),
        }
    }
    
    pub fn read<R: Read + Seek>(&self, r: &mut R) -> AnyResult<T> {
        Ok(T::parse_node(self.node.clone(), r)?)
    }
} 

impl<T: NodeParser> NodeParser for LazyData<T> {
    fn parse_node<R: Read + Seek>(raw: LevelNode, _r: &mut R) -> AnyResult<Self> {
        Ok(Self::new(raw))
    }
}

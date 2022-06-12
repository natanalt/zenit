use super::LevelNode;
use std::{
    io::{Read, Seek},
    marker::PhantomData,
};
use zenit_utils::{packed::PackedParser, AnyResult};

pub trait FromNode
where
    Self: Sized,
{
    /// Parses `Self` from given node, using provided reader. Initial seek position is undefined.
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self>;

    /// Attempts to read a node with a header.
    fn from_reader(r: &mut (impl Read + Seek)) -> AnyResult<Self> {
        let header = LevelNode::parse_header(r)?;
        Self::from_node(header, r)
    }
}

impl FromNode for LevelNode {
    fn from_node<R: Read + Seek>(raw: LevelNode, _: &mut R) -> AnyResult<Self> {
        Ok(raw)
    }
}

impl<T: PackedParser> FromNode for T {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        raw.seek_to(r)?;
        Ok(Self::parse_packed(&mut r.take(raw.payload_size as u64))?)
    }
}

/// Represents data that is not meant to be immediately loaded from its node. Instead, its
/// metadata is cached, and available for reading on demand later on. Useful when loading
/// large nodes whose binary payloads are meant to be selectively loaded and/or ignored.
#[derive(Debug, Clone)]
pub struct LazyData<T: FromNode> {
    pub node: LevelNode,
    phantom: PhantomData<T>,
}

impl<T: FromNode> LazyData<T> {
    pub fn new(node: LevelNode) -> Self {
        Self {
            node,
            phantom: Default::default(),
        }
    }

    pub fn read<R: Read + Seek>(&self, r: &mut R) -> AnyResult<T> {
        Ok(T::from_node(self.node.clone(), r)?)
    }
}

impl<T: FromNode> FromNode for LazyData<T> {
    fn from_node<R: Read + Seek>(raw: LevelNode, _r: &mut R) -> AnyResult<Self> {
        Ok(Self::new(raw))
    }
}

use crate::node::LevelNode;
use std::io::{Read, Seek};

pub trait NodeParser
where
    Self: Sized,
{
    fn parse<R: Read + Seek>(root: LevelNode, r: &mut R) -> anyhow::Result<Self>;
}

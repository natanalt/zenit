use super::LevelData;
use anyhow::bail;
use std::io::{Read, Seek};
use crate::{FromNode, LevelNode};
use zenit_utils::AnyResult;

/// Level data pack nodes are `lvl_` nodes present in certain level files
/// (like side/all.lvl for rebel alliance assets), which can be loaded
/// dynamically by the game as needed.
#[derive(Debug, Clone)]
pub struct LevelDataPack {
    /// FNV-1a hash of the name
    pub name_hash: u32,
    pub contents: LevelData,
}

impl FromNode for LevelDataPack {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        let children = raw.parse_children(r)?;
        let root = if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            bail!("invalid level data pack node contents")
        };

        Ok(Self {
            name_hash: root.name.into(),
            contents: LevelData::from_node(root, r)?,
        })
    }
}

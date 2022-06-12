use crate::LevelData;
use anyhow::bail;
use std::io::{Read, Seek};
use zenit_lvl_core::{FromNode, LevelNode};
use zenit_utils::AnyResult;

// TODO: think of a better name for optional level data nodes

/// Optional level data nodes are `LVL_` nodes present in certain level files
/// (like side/all.lvl for rebel alliance assets), which can be loaded
/// dynamically by the game as needed.
#[derive(Debug, Clone)]
pub struct OptionalLevelData {
    /// FNV-1a hash of the name
    pub name_hash: u32,
    pub contents: LevelData,
}

impl FromNode for OptionalLevelData {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        let children = raw.parse_children(r)?;
        let root = if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            bail!("invalid optional level data node contents")
        };

        Ok(Self {
            name_hash: root.name.into(),
            contents: LevelData::from_node(root, r)?,
        })
    }
}

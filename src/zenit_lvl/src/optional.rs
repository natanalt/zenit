use crate::LevelData;
use anyhow::bail;
use std::io::{Read, Seek};
use zenit_lvl_core::{FromNode, LevelNode};
use zenit_utils::AnyResult;

#[derive(Debug, Clone)]
pub struct OptionalLevelData {
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

use crate::{
    node, ConfigData, ConfigObject, ConfigScope, FromNode, LevelConfig, LevelNode, NodeName,
};
use anyhow::{anyhow, bail};
use byteorder::{ReadBytesExt, LE};
use std::io::{self, Read, Seek};
use zenit_utils::AnyResult;

impl FromNode for LevelConfig {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        let mut name_hash = None;
        let mut root = ConfigScope::default();
        let children = &mut root.children;

        for child in raw.iter_children(r)? {
            if child.name == node!("NAME") {
                if name_hash.is_some() {
                    bail!("duplicate name in config node");
                }
                name_hash = Some(child.data_reader(r)?.read_u32::<LE>()?);
            } else if child.name == node!("DATA") {
                children.push(ConfigObject::Data(ConfigData::from_node(child, r)?))
            } else if child.name == node!("SCOP") {
                children.push(ConfigObject::Scope(ConfigScope::from_node(child, r)?))
            }
        }

        Ok(Self {
            name_hash: name_hash.ok_or(anyhow!("missing config node name"))?,
            root,
        })
    }
}

impl FromNode for ConfigScope {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        let mut children = vec![];
        for child in raw.iter_children(r)? {
            if child.name == node!("DATA") {
                children.push(ConfigObject::Data(ConfigData::from_node(child, r)?))
            } else if child.name == node!("SCOP") {
                children.push(ConfigObject::Scope(ConfigScope::from_node(child, r)?))
            }
        }
        Ok(Self { children })
    }
}

impl FromNode for ConfigData {
    fn from_node<R: Read + Seek>(raw: LevelNode, r: &mut R) -> AnyResult<Self> {
        let mut data = raw.data_reader(r)?;

        let name_hash = data.read_u32::<LE>()?;

        let value_count = data.read_u8()?;
        let values = (0..value_count)
            .map(|_| Ok(data.read_u32::<LE>()?))
            .collect::<io::Result<Vec<u32>>>()?;

        let tail_length = data.read_u32::<LE>()?;
        let mut tail = vec![0; tail_length as usize];
        data.read_exact(&mut tail)?;

        Ok(Self {
            name_hash,
            values,
            tail,
        })
    }
}

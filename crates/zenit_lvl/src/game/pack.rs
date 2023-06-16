use super::LevelData;
use crate::node::{read_node_children, NodeHeader, NodeRead, NodeWrite, NodeWriter};
use anyhow::bail;
use std::io::{Read, Seek, Write};
use zenit_utils::{ok, AnyResult};

/// Level data pack nodes are `lvl_` nodes present in certain level files
/// (like side/all.lvl for rebel alliance assets), which can be loaded
/// dynamically by the game as needed.
#[derive(Debug, Clone)]
pub struct LevelDataPack {
    /// FNV-1a hash of the name
    pub name_hash: u32,
    pub contents: LevelData,
}

impl NodeRead for LevelDataPack {
    fn read_node_payload<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        let children = read_node_children(r, meta)?;
        let root = if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            bail!("invalid level data pack node contents")
        };

        Ok(Self {
            name_hash: root.name.into(),
            contents: LevelData::read_node_payload(r, root)?,
        })
    }
}
impl NodeWrite for LevelDataPack {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        // TODO: get rid of the clone once PackedWrite is split into Read/Write traits
        writer.write_node(self.name_hash, self.contents.clone())?;
        ok()
    }
}

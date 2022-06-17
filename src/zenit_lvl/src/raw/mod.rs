use pack::LevelDataPack;
use script::LevelScript;
use texture::LevelTexture;
use zenit_proc::FromNode;

pub mod model;
pub mod pack;
pub mod script;
pub mod texture;

/// Main representation of a level file.
///
/// You can read it using the [`FromNode::from_reader`] function.
#[derive(Debug, Clone, FromNode)]
pub struct LevelData {
    #[nodes("lvl_")]
    pub packs: Vec<LevelDataPack>,
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,
}

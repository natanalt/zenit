use optional::OptionalLevelData;
use script::LevelScript;
use texture::LevelTexture;
use zenit_proc::FromNode;

pub mod optional;
pub mod script;
pub mod texture;
pub mod model;

/// Main representation of a level file.
/// 
/// You can read it using the [`FromNode::from_reader`] function.
#[derive(Debug, Clone, FromNode)]
pub struct LevelData {
    #[nodes("lvl_")]
    pub hashed: Vec<OptionalLevelData>,
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,
}


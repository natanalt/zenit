use optional::OptionalLevelData;
use script::LevelScript;
use texture::LevelTexture;
use zenit_proc::FromNode;

pub mod optional;
pub mod script;
pub mod texture;

#[derive(Debug, Clone, FromNode)]
pub struct LevelData {
    #[nodes("lvl_")]
    pub hashed: Vec<OptionalLevelData>,
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,
}

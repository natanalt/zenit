use script::LevelScript;
use texture::LevelTexture;
use zenit_proc::FromNode;

pub mod script;
pub mod texture;

#[derive(Debug, Clone, FromNode)]
pub struct LevelData {
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,
}

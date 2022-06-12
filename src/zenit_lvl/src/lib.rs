//! Parsing library for *Star Wars Battlefront II* level files.
//! 
//! This library only lets you access raw data in an accessible and portable
//! way. For example, textures aren't decoded and are only exposed as raw
//! byte buffers.
//! 
//! `zenit_lvl` isn't dependent on main engine itself and so can be easily used
//! anywhere else (it still has dependencies on a few utility crates within
//! Zenit, see the crate manifest's dependencies for details.)

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

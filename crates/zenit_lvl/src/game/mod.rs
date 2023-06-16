//! Base definitions of SWBF2 node types

use crate::node::*;

mod model;
pub use model::*;
mod pack;
pub use pack::*;
mod script;
pub use script::*;
mod texture;
pub use texture::*;

/// Main representation of a level file.
///
/// You can read it using the [`ReadNode::from_reader`] function.
#[derive(Debug, Clone, NodeData)]
pub struct LevelData {
    #[nodes("lvl_")]
    pub packs: Vec<LevelDataPack>,
    #[nodes("scr_")]
    pub scripts: Vec<LevelScript>,
    #[nodes("tex_")]
    pub textures: Vec<LevelTexture>,

    #[cfg(feature = "zenit_extensions")]
    #[nodes("WGSL")]
    pub wgsl_shaders: Vec<crate::zext::LevelWgslShader>,
}


pub mod loader;
pub mod munge_asset_io;
pub mod munge;
pub mod texture;

pub use munge_asset_io::MungeAssetIoPlugin;

use self::munge::MungeName;

#[derive(Debug, Clone, Copy)]
pub enum AssetKind {
    Script,
    Texture,
}

impl AssetKind {
    pub fn extension(&self) -> &'static str {
        match self {
            AssetKind::Script => "luac",
            AssetKind::Texture => "ztexture",
        }
    }

    pub fn from_node_name(n: MungeName) -> Option<AssetKind> {
        match n.to_string().as_str() {
            "scr_" => Some(AssetKind::Script),
            "tex_" => Some(AssetKind::Texture),
            _ => None,
        }
    }

    pub fn from_extension(s: &str) -> Option<AssetKind> {
        match s {
            "luac" => Some(AssetKind::Script),
            "ztexture" => Some(AssetKind::Texture),
            _ => None,
        }
    }
}

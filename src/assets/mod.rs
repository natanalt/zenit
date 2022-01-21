
pub mod loader;
pub mod munge_asset_io;
pub mod munge;
pub mod texture;
pub mod script;

pub use munge_asset_io::MungeAssetIoPlugin;

use self::munge::MungeName;

#[derive(Debug, Clone, Copy)]
pub enum AssetKind {
    Script,
    Texture,
    Unsupported,
}

impl AssetKind {
    pub fn extension(&self) -> &'static str {
        match self {
            AssetKind::Script => "luac",
            AssetKind::Texture => "ztexture",
            AssetKind::Unsupported => "unsupported",
        }
    }

    pub fn from_node_name(n: MungeName) -> AssetKind {
        let stringified: String = n.try_into().expect("invalid asset node name");
        match stringified.as_str() {
            "scr_" => AssetKind::Script,
            "tex_" => AssetKind::Texture,
            _ => AssetKind::Unsupported,
        }
    }

    pub fn from_extension(s: &str) -> AssetKind {
        match s {
            "luac" => AssetKind::Script,
            "ztexture" => AssetKind::Texture,
            _ => AssetKind::Unsupported,
        }
    }
}

mod texture;

use bevy::{prelude::*, asset::{AssetLoader, BoxedFuture, LoadContext}, app::PluginGroupBuilder};
use crate::AnyResult;

use super::munge::MungeName;

pub const LOADABLE_NODES: &'static [MungeName] = &[
    MungeName::from_literal("scr_"),
    MungeName::from_literal("tex_"),
];

pub fn is_loadable(name: MungeName) -> bool {
    LOADABLE_NODES.contains(&name)
}

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

/// Loader of munged (.lvl) files.
#[derive(Default)]
pub struct MungeLoader;

impl AssetLoader for MungeLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, AnyResult<()>> {
        info!("First bytes: {:x} {:x} {:x} {:x}", bytes[0], bytes[1], bytes[2], bytes[3]);
        let metas = load_context.get_asset_metas();
        info!("{}", metas.len());
        for meta in load_context.get_asset_metas() {
            info!("{:#?}", meta.label);
        }
        info!("Loading {:#?}", load_context.path().to_str());
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &["lvl"]
    }
}

pub struct LoaderPlugins;

impl PluginGroup for LoaderPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        
    }
}

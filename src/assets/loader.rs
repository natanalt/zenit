use bevy::{prelude::*, asset::{AssetLoader, BoxedFuture, LoadContext}};
use crate::AnyResult;

use super::munge::MungeName;

pub const LOADABLE_NODES: &'static [MungeName] = &[
    MungeName::from_literal("scr_"),
    MungeName::from_literal("tex_"),
];

pub fn is_loadable(name: MungeName) -> bool {
    LOADABLE_NODES.contains(&name)
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

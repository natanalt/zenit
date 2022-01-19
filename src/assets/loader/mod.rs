use self::texture::MungeTextureLoaderPlugin;
use super::munge::MungeName;
use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod texture;

pub const LOADABLE_NODES: &'static [MungeName] = &[
    MungeName::from_literal("scr_"),
    MungeName::from_literal("tex_"),
];

pub fn is_loadable(name: MungeName) -> bool {
    LOADABLE_NODES.contains(&name)
}

pub struct LoaderPlugins;

impl PluginGroup for LoaderPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MungeTextureLoaderPlugin);
    }
}

//! Zenit Engine entry point
use args::ZenitArgs;
use assets::{MungeAssetIoPlugin, loader::LoaderPlugins};
use bevy::{asset::AssetPlugin, prelude::*};
use clap::StructOpt;

pub mod args;
pub mod assets;
pub mod lua;
pub mod utils;
pub mod loading;

pub type AnyResult<T, E = anyhow::Error> = anyhow::Result<T, E>;

fn main() {
    App::new()
        .insert_resource(ZenitArgs::parse())
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<AssetPlugin, _>(MungeAssetIoPlugin)
        })
        .add_plugins(LoaderPlugins)
        .run();
}

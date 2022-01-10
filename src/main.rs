//! Zenit Engine entry point
use bevy::prelude::*;
use clap::StructOpt;

pub mod args;
pub mod assets;
pub mod lua;
pub mod utils;

pub type AnyResult<T, E = anyhow::Error> = anyhow::Result<T, E>;

fn main() {
    App::new()
        .insert_resource(args::ZenitArgs::parse())
        .add_plugins(DefaultPlugins)
        .run();
}

use std::path::PathBuf;
use clap::Parser;

/// All command line parameters that can be passed to Zenit. Stored within the Bevy ECS as
/// a resource, so usually you can get an instance from there.
#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct ZenitArgs {
    /// Path to SWBF2's root directory (one containing the `GameData` folder)
    #[clap(long)]
    pub game_root: PathBuf,
}

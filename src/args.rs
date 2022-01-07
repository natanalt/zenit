use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct ZenitArgs {
    /// Path to SWBF2's root directory (one containing the `GameData` folder)
    #[clap(long)]
    pub game_root: PathBuf,
}

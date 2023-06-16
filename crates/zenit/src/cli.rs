use clap::Parser;
use std::path::PathBuf;

/// User-specified command line parameters
#[derive(Debug, Parser)]
#[clap(name = "Zenit Engine", about)]
pub struct Args {
    #[clap(long, short = 'r')]
    /// Overrides the path to SWBF2's game root (with a `GameData` directory).
    pub game_root: Option<PathBuf>,

    #[clap(long)]
    /// Forces the engine to run singlethreaded. You should generally keep this off.
    pub single_thread: bool,
}

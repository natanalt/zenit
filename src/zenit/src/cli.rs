use clap::Parser;
use zenit_proc::Data;
use std::path::PathBuf;

/// User-specified command line parameters
#[derive(Parser)]
#[clap(name = "Zenit Engine", about)]
pub struct Args {
    #[clap(
        long,
        short = 'r',
        help = "Overrides the path to SWBF2's game root (with a `GameData` \
                directory). On Windows, Zenit attempts to automatically detect \
                it. If none is available, user will be prompted for it after \
                start."
    )]
    pub game_root: Option<PathBuf>,
}

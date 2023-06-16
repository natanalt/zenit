use clap::{Args, Subcommand};
use std::path::PathBuf;
use zenit_lvl::game::D3DFormat;

#[derive(Subcommand)]
pub enum AppendCommand {
    /// Includes a 2D texture in the data file.
    Texture(TextureInclude),
}

#[derive(Args)]
pub struct TextureInclude {
    /// Path to the output data file.
    ///
    /// If this file doesn't exist, it'll be created.
    pub output_path: PathBuf,
    /// Path to the texture within the data file.
    pub texture_path: String,

    #[arg(long, short = 'o')]
    pub formats: Vec<D3DFormat>,
}

impl crate::Command for AppendCommand {
    fn run(self) -> zenit_utils::AnyResult {
        todo!()
    }
}

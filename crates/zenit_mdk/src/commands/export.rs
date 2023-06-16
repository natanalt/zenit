use clap::{Args, Subcommand};
use std::path::PathBuf;
use zenit_lvl::game::D3DFormat;
use zenit_utils::AnyResult;

#[derive(Subcommand)]
pub enum ExportCommand {
    /// Exports a 2D texture.
    Texture(TextureExport),
    /// Exports a 2D cubemap.
    Cubemap,
}

#[derive(Args)]
pub struct TextureExport {
    /// Path to the data file.
    pub file_path: PathBuf,
    /// Path to the texture.
    ///
    /// To export the texture from a `lvl_` node, path can be separated with a `/`.
    pub path: String,
    /// Texture format to use. If only one format is present, it can be omitted.
    pub format: Option<D3DFormat>,
    /// Mip level to extract.
    #[arg(long, default_value_t = 0)]
    pub mipmap: u32,
}

impl crate::Command for ExportCommand {
    fn run(self) -> AnyResult {
        todo!()
    }
}

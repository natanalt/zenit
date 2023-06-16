use crate::exporter::texture::{export_texture, TextureSpecification};
use clap::Args;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};
use zenit_lvl::node::NodeWriter;
use zenit_utils::{ok, AnyResult};

#[derive(Args)]
pub struct BuildCommand {
    /// Output file
    #[clap(long, short = 'o')]
    pub output: PathBuf,
    /// Specification file to use
    pub specification: PathBuf,
}

impl crate::Command for BuildCommand {
    fn run(self) -> AnyResult {
        let spec_text = match fs::read_to_string(self.specification) {
            Ok(specs) => specs,
            Err(err) => {
                eprintln!("An error occured while reading the specification: {err:#?}");
                return Err(err.into());
            }
        };

        let spec = match toml::from_str::<PackSpecification>(&spec_text) {
            Ok(spec) => spec,
            Err(err) => {
                eprintln!("An error occurred while parsing the specification: {err:#?}");
                return Err(err.into());
            }
        };

        let mut file = BufWriter::new(
            File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(self.output)?,
        );
        let mut writer = NodeWriter::new(&mut file, b"ucfb")?;

        println!(" : Writing textures...");
        for texture in spec.textures {
            println!("  - Writing {}...", texture.name);
            writer.write_node(b"tex_", export_texture(texture)?)?;
        }

        println!(" : Finishing...");
        writer.finish()?;

        ok()
    }
}

#[derive(Debug, Deserialize)]
struct PackSpecification {
    textures: Vec<TextureSpecification>,
}

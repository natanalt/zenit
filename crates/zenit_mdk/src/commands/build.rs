use crate::exporter::{shader::ShaderSpecification, texture::TextureSpecification};
use clap::Args;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
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
            writer.write_node(b"tex_", texture.export()?)?;
        }

        println!(" : Writing shaders...");

        println!("  - Reading shared definitions...");
        let mut shader_shared = String::new();
        for dir in spec.shader_preprocessor.shared {
            for dir_entry_result in fs::read_dir(&dir)? {
                let dir_entry = dir_entry_result?;

                if !dir_entry.file_type()?.is_file() {
                    continue;
                }

                let file_name = dir_entry.file_name();
                let path = dir.join(Path::new(&file_name));
                //if !path.ends_with(".inc.wgsl") {
                //    continue;
                //}

                println!("    - Including {path:?}...");

                shader_shared.push_str(&fs::read_to_string(path)?);
                shader_shared.push('\n');
            }
        }

        for shader in spec.shaders {
            println!("  - Writing {}...", shader.name);
            writer.write_node(b"WGSL", shader.export(&shader_shared)?)?;
        }

        println!(" : Finishing...");
        writer.finish()?;

        ok()
    }
}

#[derive(Debug, Deserialize)]
struct PackSpecification {
    textures: Vec<TextureSpecification>,
    shaders: Vec<ShaderSpecification>,
    #[serde(default)]
    shader_preprocessor: ShaderPreprocessorSettings,
}

#[derive(Debug, Default, Deserialize)]
struct ShaderPreprocessorSettings {
    shared: Vec<PathBuf>,
}

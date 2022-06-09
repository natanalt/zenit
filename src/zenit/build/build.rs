use std::{env, ffi::OsString, fs, io};

mod shaders;
use shaders::*;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.rc");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.exe.manifest");

    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("src/platform/windows/zenit.rc");
    }

    let toolchain = GlslToolchain::find();

    for entry in fs::read_dir("assets/shaders")? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_file() {
            continue;
        }

        if path.extension() != Some(&OsString::from("shader")) {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy();
        println!("Compiling shader `{}`...", &name);

        let source = ShaderSource::parse(&path)?;

        ShaderCompilation::builder()
            .toolchain(&toolchain)
            .name(name.to_string())
            .metadata(source.metadata)
            .vertex_shader(format!("{}\n{}", source.shared, source.vertex))
            .fragment_shader(format!("{}\n{}", source.shared, source.fragment))
            .build()
            .unwrap()
            .invoke()?;
    }

    Ok(())
}

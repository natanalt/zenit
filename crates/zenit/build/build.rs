use std::{env, ffi::OsString, fs, io};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=crates/platform/windows/zenit.rc");
    println!("cargo:rerun-if-changed=crates/platform/windows/zenit.exe.manifest");

    let mut vergen_config = vergen::Config::default();
    *vergen_config.build_mut().kind_mut() = vergen::TimestampKind::All;
    vergen::vergen(vergen_config).unwrap();

    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("src/platform/windows/zenit.rc");
    }

    Ok(())
}

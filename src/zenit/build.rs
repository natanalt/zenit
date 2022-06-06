use std::env;

fn main() {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.rc");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.exe.manifest");

    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("src/platform/windows/zenit.rc");
    }
}

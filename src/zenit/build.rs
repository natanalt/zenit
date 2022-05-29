use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.rc");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.exe.manifest");

    if let Ok(_) = env::var("CARGO_CFG_WINDOWS") {
        embed_resource::compile("src/platform/windows/zenit.rc");
    }
}

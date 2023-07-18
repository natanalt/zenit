use std::{env, io, path::PathBuf};
use zenit_mdk::{commands::build::BuildCommand, CliCommand};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.rc");
    println!("cargo:rerun-if-changed=src/platform/windows/zenit.exe.manifest");

    println!(" == Running vergen...");
    vergen::EmitBuilder::builder()
        .git_branch()
        .git_sha(true)
        .rustc_semver()
        .rustc_channel()
        .build_timestamp()
        .emit()
        .unwrap();

    println!(" == Running resource embedding...");
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("src/platform/windows/zenit.rc");
    }

    println!(" == Building zenit_builtin.lvl...");
    let old_cwd = env::current_dir()?;
    env::set_current_dir(old_cwd.join("assets/builtin"))?;
    zenit_mdk::run(zenit_mdk::Cli {
        command: CliCommand::Build(BuildCommand {
            output: PathBuf::from(env::var("OUT_DIR").unwrap()).join("zenit_builtin.lvl"),
            specification: PathBuf::from("zenit_builtin.toml"),
        }),
    })
    .expect("builtin asset build failed");
    env::set_current_dir(old_cwd)?;

    println!(" == All done on zenit/build.rs side!");
    Ok(())
}

use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=lua/src");
    println!("cargo:rerun-if-changed=lua/src/lib");
    println!("cargo:rerun-if-changed=lua/wrapper.h");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Create the Lua bindings
    bindgen::Builder::default()
        .clang_arg("-Ilua/include")
        .clang_arg("-DTRUST_BINARIES")
        .clang_arg("-DLUA_NUMBER=float")
        .header("lua/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("couldn't generate Lua bindings")
        .write_to_file(out_dir.join("lua_bindings.rs"))
        .expect("couldn't write output Lua bindings");

    // Build the Lua static library
    cc::Build::new()
        .include("./lua/include")
        .define("TRUST_BINARIES", None)
        .define("LUA_NUMBER", "float")
        .files(
            glob::glob("./lua/src/**/*.c")
                .expect("couldn't glob Lua sources")
                .map(|x| x.unwrap()),
        )
        .compile("lua");
}

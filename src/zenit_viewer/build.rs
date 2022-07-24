use std::env;

fn main() {
    // Now that's what I call a big hack!
    // HACK: GTK on Windows is a hack.
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!(r"cargo:rustc-link-search=C:\gtk-build\gtk\x64\release\lib");
    }
}

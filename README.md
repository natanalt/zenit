# 🚀 Zenit Engine
[![Unit tests (Windows)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml/badge.svg)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml)
[![Unit tests (Linux)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-linux.yml/badge.svg)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-linux.yml)

**Zenit** is a project attempting to create an open-source engine compatible with the PC version of *Star Wars Battlefront II (2005)*.

Unlike other projects, like [Phoenix](https://github.com/LibSWBF2/SWBF2Phoenix), Zenit tries to stay more faithful to the original game's look and feel, while also being fully portable. This doesn't mean that it'll never allow any graphical fireworks, but its priority is to at the very least look like the original. Zenit and and all of its dependencies are also fully open-source, if that's your thing (even if it does lose on graphical goodies of Unity, which are used in Phoenix).

It's still an extreme work in progress, and given my constant rewrite habits, it'll stay in this state for a while.

## Building
Requirements:
 * Patience
 * The latest stable version of the [Rust toolchain](https://rust-lang.org)
 * A bunch more packages are needed for Linux builds, idk I'll update this someday

The usual development workflow is:
 * `cargo build` to build the project
 * `cargo run` to run it (use `cargo run -- parameters` to pass command line arguments)
 * `cargo build --bin crate_name` to only build a single crate
 * *(using `--profile release` is generally recommended)*

You can also look at automated unit test workflows in the [*.github/workflows*](.github/workflows) directory.

## Requirements
Zenit requires slightly newer hardware than the original PC game. My work machine uses a 4th gen Intel i3 + GTX 750 Ti, so it's not super demanding!

### Windows
 * Windows 10 or higher
   - Versions as low as Windows 7 may work, but no guarantees here
 * DirectX 12 or Vulkan 1.1 compatible graphics card
   - OpenGL *may* work, DirectX 11 support is dependent on the [wgpu](https://github.com/gfx-rs/wgpu) library used by Zenit

### Linux
 * No specifics on the Linux kernel version, but you play games on Linux, you have to use a fairly recent kernel.
 * Vulkan 1.1 or higher, OpenGL ES 3 may work
 * Linux builds require additional library packages for the build.
   - TODO: list the necessary packages

### Other platforms
Other platforms, like macOS and Android will be supported in the future. macOS builds may already work with minimal changes, but I don't have any Macs at the moment, so I can't test this. Linux and Windows builds are automatically CI tested, so those are the only guarantees I can make.

## Internal project structure
The project is separated into multiple crates in the [crates](crates/) directory:
 * [**crates/zenit**](crates/zenit/) - the main engine and the core of Zenit's codebase
 * [**crates/zenit_lvl**](crates/zenit_lvl/) - loader of BF2's level files, can be used as a standalone library
 * [**crates/zenit_lua**](crates/zenit_lua/) - Lua 5.0.2 bindings and a custom architecture independent x86-32 chunk loader
 * [**crates/zenit_utils**](crates/zenit_utils/) - general utilities and shared code
 * [**crates/zenit_proc**](crates/zenit_proc/) - engine-wide proc macros

Stuff that's not currently there but may be added in the future:
 * **crates/zenit_mdk** - mod development kit, aka. an executable for generating munge files 

### Format reference book
The directory **format_book/** contains [mdBook](https://github.com/rust-lang/mdBook)-based documentation for Battlefront II's file formats. It's better served there, than in random comments in the project. It's not yet automatically deployed online.

If you have mdBook installed (`cargo install mdbook`) you can also run `mdbook serve` from the format book's directory, to host a local HTTP server with a pretty, viewable version of the documentation.

## License stuff
No license attached here yet; I'll add one someday lol (that does technically make the game not actually open-source, but come on)

At this point, the project comes with Lua 5.0.2, see [`src/zenit_lua/lua/COPYRIGHT`] for details.

Also, of course, I am not affiliated with the late Pandemic Studios, Disney or any other legal entities or individuals that may hold any rights to the original game, or the entire Star Wars franchise. This is merely a fan project to a game I feel a lot of nostalgia for. It doesn't enable piracy, and it never will.

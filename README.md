# 🚀 Zenit Engine
<!--
Unit tests need modification to work again, see unit-tests-windows.yml

[![Unit tests (Windows)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml/badge.svg)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml)
-->
**Zenit** is a project attempting to create an open-source engine compatible with the PC version of *Star Wars Battlefront II (2005)*.

Unlike the impressive [Phoenix](https://github.com/LibSWBF2/SWBF2Phoenix) project, Zenit tries to stay more faithful to the original game's look and feel, while also being fully portable. This doesn't mean that it'll never allow any graphical fireworks, but its priority is to at the very least look like the original. Zenit and and all of its dependencies are also open-source, if that's your thing (even if it does lose on graphical goodies of Unity, which are used in Phoenix).

It's still super early in its life and you can't even start it yet. Work is underway to make it at least load to menu, though. lol

## Building
Requirements:
 * Recent version of the [Rust toolchain](https://rust-lang.org)
 * `glslc` and `spirv-link`
    * the build scripts look for them either in your PATH or in `$VULKAN_SDK/bin`
 * Patience

The usual workflow is:
 * `cargo build` to build the project
 * `cargo run` to run it (use `cargo run -- parameters` to pass command line arguments)
 * `cargo build --bin crate_name` to only build a single crate

You can also look at automated unit test workflows in the [*.github/workflows*](.github/workflows) directory.

## Internal project structure
The project is separated into multiple crates in the src directory:
 * **src/zenit_utils** - general utilities and shared code
 * **src/zenit_lua** - Lua 5.0.2 bindings and a custom architecture independent x86-32 chunk loader
 * **src/zenit_lvl** - loader of BF2's level files, can be used as a standalone library
 * **src/zenit_lvl_core** - simple core of the level file reader, without any game-specific definitoins
 * **src/zenit_proc** - engine-wide proc macros
 * **src/zenit** - the main engine and the core of Zenit's codebase

Stuff that's not currently there but may be added in the future:
 * **src/zenit_mdk** - mod development kit, aka. an executable for generating munge files 

### Format reference book
The directory **format_book/** contains [mdBook](https://github.com/rust-lang/mdBook)-based documentation for Battlefront II's file formats. It's better served there, than in random comments in the project. It's not yet automatically deployed online.

If you have mdBook installed (`cargo install mdbook`) you can also run `mdbook serve` from the format book's directory, to host a local HTTP server with a pretty, viewable version of the documentation.

## License stuff
No license attached here yet; I'll finally add it someday lol

This project also comes with Lua 5.0.2, see [`src/zenit_lua/lua/COPYRIGHT`] for details.

Also, of course, I am not affiliated with the late Pandemic Studios, Disney or any other legal entities or individuals that may have any rights to the original game. This is merely a fan project to a game I feel a lot of nostalgia for. I'm not trying to breach any copyright laws here.

# 🚀 Zenit Engine
[![Unit tests (Windows)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml/badge.svg)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-windows.yml) [![Unit tests (Linux)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-linux.yml/badge.svg)](https://github.com/natanalt/zenit/actions/workflows/unit-tests-linux.yml)

Zenit is a project attempting to create an open-source engine compatible with data files of PC version of *Star Wars Battlefront II (2005)*.

Unlike the impressive [Phoenix](https://github.com/LibSWBF2/SWBF2Phoenix) project, Zenit tries to stay more faithful to the original game's look and feel, while also being fully portable. This doesn't mean that it'll never allow any graphical fireworks, but its priority is to at the very least look like the original. Zenit and and all of its dependencies are also open-source, if that's your thing (even if it does lose on graphical goodies of Unity, which are used in Phoenix).

It's still super early in its life and you can't even start it yet. Work is underway to make it at least load to menu, though. lol

## Building
To build Zenit you need Rust and a C compiler compatible with the toolchain (used to compile Lua). Just run `cargo build`. It should work on Windows and Linux without issues.

You can also look at automated unit test workflows in the [.github/workflows](.github/workflows) directory. Note, that I actively work on Windows as of now, so Linux builds are completely untested.

### macOS support
Zenit should be compatible with macOS, but not in its current form. Since the renderer is using Vulkan, additional setup will be required, to integrate a compatibility library like MoltenVK. Unfortunately I don't own a Mac at the moment, so I can't really do that.

## Internal project structure
The project is separated into multiple crates in the src directory:
 * **src/zenit_utils** - general utilities and shared code
 * **src/zenit_lua** - Lua 5.0.2 bindings and a custom architecture independent x86-32 chunk loader
 * **src/zenit_lvl** - loader of BF2's level files, can be used as a standalone library
 * **src/zenit_lvl_core** - simple core of the level file reader, without any game-specific definitoins
 * **src/zenit_proc** - engine-wide proc macros
 * **src/zenit_render** - custom renderer engine for Zenit
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

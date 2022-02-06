# 🚀 Zenit Engine
Zenit is a project attempting to create an open-source engine compatible with data files *Star Wars Battlefront II (2005)'s* PC version.

Unlike the impressive [Phoenix](https://github.com/LibSWBF2/SWBF2Phoenix) project, Zenit tries to stay more faithful to the original game's look and feel, while also being fully portable. This doesn't mean that it'll never allow any graphical fireworks, but its priority is to at the very least look like the original. Zenit and and all of its dependencies are also open-source, if that's your thing (even if it does lose on graphical goodies of Unity, which are used in Phoenix).

It's still super early in its life and you can't even start it yet. Work is underway to make it at least load to menu, though. lol

## Building
You need a nightly build of the Rust compiler (`rustup toolchain add nightly`) and a C compiler like MSVC, GCC, or Clang. Running `cargo build` will build all Zenit crates.

## Internal project structure
The project is separated into multiple crates in the src directory:
 * **src/zenit_utils** - general utilities and shared code
 * **src/zenit_lua** - Lua 5.0.2 bindings and a custom architecture independent x86-32 chunk loader
 * **src/zenit_lvl** - loader of BF2's level files, not dependent on anything besides utilities
 * **src/zenit** - the main engine and the core of Zenit's codebase

Stuff that's not currently there but may be added in the future:
 * **src/zenit_mdk** - mod development kit, aka. an executable for generating munge files 

## License stuff
No license attached here yet; it'll likely be GPL, I'll finally add it someday lol

This project also comes with Lua 5.0.2, see `creates/zenit_lua/lua/COPYRIGHT` file for details.

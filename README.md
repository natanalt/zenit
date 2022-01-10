# Zenit Engine
Zenit is a project attempting to create an open-source engine, compatible with data files of PC edition of *Star Wars Battlefront II (2005)*.

It's still super early in its life and you can't even start it yet. Work is underway to make it at least load to menu, though. lol

## Runtime requirements
 * A legal copy of PC SWBF2.
   DVD retail copies also work (I personally use one), just make sure to install the game properly first, to unpack the data.
   If you don't own a copy, you could get it on [Steam](https://store.steampowered.com/app/6060/Star_Wars_Battlefront_2_Classic_2005/)
 * a computer idk

## Building
You need two things:
 * a Rust compiler (unstable)
 * a C compiler usable by Rust, such as MSVC or GCC

Run `cargo build` and wait for magic to happen. The first build will spiral to a LOT, mostly because of Bevy. The build is configured to compile Bevy into a DLL, so after the first build you shouldn't have to wait for it to build again.

## License stuff
No license attached here yet; it'll likely be GPL

This project also comes with Lua 5.0.2, see `lua/COPYRIGHT` file for details.

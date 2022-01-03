# Zenit Engine
Zenit is a project attempting to create an open-source engine, compatible with data files of PC edition of *Star Wars Battlefront II (2005)*.

It's still super early in its life and you can't even start it yet. Work is underway to make it at least load to menu, though. lol

## Building
You need two things:
 * a Rust compiler (latest stable)
 * a C compiler usable by Rust, such as MSVC or GCC
 * actual legal copy of the game would be handy, you can get it on [Steam](https://store.steampowered.com/app/6060/Star_Wars_Battlefront_2_Classic_2005/), or a DVD retail (like I do)

Run `cargo build` and wait for magic to happen. Keep in mind, that the dependencies may spiral into like 200 needed libraries, which is, uh...

## License stuff
No license attached here yet; it'll likely be GPL

This project also comes with Lua 5.0.2, see `lua/COPYRIGHT` file for details.

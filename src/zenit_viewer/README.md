# Zenit Viewer
The viewer application is written with GTK4 in mind. On Linux and macOS, getting GTK4 libraries is trivial, on Windows - not so much.

## Getting this running under Windows
The simplest way is to build GTK using [`gvsbuild scripts`](https://github.com/wingtk/gvsbuild). Follow the procedures written down in their README, and you should end up with a directory `C:\gtk-build` containing build artifacts.

To make them visible to gtk-rs's build scripts, modify the following environment variables:
 * Add `C:\gtk-build\gtk\x64\release\bin` to `PATH`
 * Put `C:\gtk-build\gtk\x64\release\lib\pkgconfig` in `PKG_CONFIG_PATH`

*(you may have to restart your IDE and/or terminals for the changes to be visible)*

### Note
The gtk-rs build scripts seem to not pass GTK library paths properly to the linker, so the `build.rs` build script manually attaches the following one: `C:/gtk-build/gtk/x64/release/lib`. That's something to watch out for.

Another thing to watch out for is that I am assuming usage of the MSVC toolchain - getting the GNU toolchain working here would require some additional fiddling.

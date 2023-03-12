//! Crash handling code

use std::panic::PanicInfo;

// TODO: properly handle thread panics
//       A thread panicking in Zenit is going to be bad™️ regardless of where
//       it happens, but Rust's default behavior is that a panic only takes
//       down the thread where said panic happened.

#[cfg(target_os = "windows")]
mod windows;

/// Sets up the panic hook, if any is available for the current platform
#[inline(always)]
pub fn enable_panic_handler() {
    #[cfg(target_os = "windows")]
    windows::set_panic_hook();
}

pub fn generate_error_log(panic_info: &PanicInfo) -> String {
    let engine_version = crate::VERSION;
    let local_time = chrono::Local::now();
    let build_type = cfg!(debug_assertions).then(|| "Debug").unwrap_or("Release");
    let commit_hash = "<TODO>";
    let thread_name = std::thread::current()
        .name()
        .unwrap_or("(no name)")
        .to_string();
    let rust_backtrace = "    <TODO>";
    let lua_backtrace = "    <TODO>";
    let cpu = "<TODO>";
    let cores = "<TODO>";
    let gfx_name = "<TODO>";
    let gfx_backend = "<TODO>";
    let gfx_drivers = "<TODO>";
    let ram_size = "<TODO>";
    let os_name = "<TODO>";
    let loaded_level = "<TODO>";
    let loaded_files = "<TODO>";

    format!(
        r###"
```
Zenit Engine {engine_version} Crash Report

Local time: {local_time}
Build type: {build_type} {engine_version} {commit_hash}

Panic information
=================================================
 - {panic_info}
 - panicking thread: {thread_name}

Rust backtrace:
{rust_backtrace}

Lua backtrace:
{lua_backtrace}

Hardware information
=================================================
 - CPU: {cpu} ({cores})
 - GPU: {gfx_name} ({gfx_backend} {gfx_drivers})
 - RAM: {ram_size}
 - OS: {os_name}

Ingame information
=================================================
Loaded level: {loaded_level}

Asset manager:
 - loaded data files: {loaded_files}

<end of crash report>
```
    "###
    )
    .trim()
    .to_string()
}

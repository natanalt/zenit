//! Miscellaneous platform-specific code and files

use std::path::PathBuf;

// TODO: what should the code do if there are multiple installations of BF2?
//       (like retail + Steam)

// TODO: potential BF2 search functions for OSes other than Windows
//       For example, on Linux the code could look for a Steam installation in
//       the user's home folder, and see if the game is installed via Proton.

/// Tries to find an installation of SWBF2 somewhere on this computer, using
/// platform specific methods.
///
/// Returns `None` if none is found, or there's no search implementation for
/// current platform.
#[allow(unreachable_code)]
pub fn find_bf2() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        #[path = "windows/mod.rs"]
        mod windows;
        return windows::find_bf2();
    }
    None
}

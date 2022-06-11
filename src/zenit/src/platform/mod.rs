use std::path::PathBuf;

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

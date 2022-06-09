use std::panic::PanicInfo;

#[cfg(target_os = "windows")]
mod windows;

/// Sets up the panic hook, if any is available for the current platform
#[inline(always)]
pub fn enable_panic_handler() {
    #[cfg(target_os = "windows")]
    windows::set_panic_hook();
}

pub fn generate_error_log(info: &PanicInfo) -> String {
    format!(
        "Zenit Engine {} Crash Report\n\n\
        \
        Local date: {}\n\
        Build type: {}\n\n\
        \
        {}\n\n\
        \
        TODO: better crash logs",
        crate::VERSION,
        chrono::Local::now(),
        cfg!(debug_assertions).then(|| "Debug").unwrap_or("Release"),
        info.to_string(),
    )
}

use log::*;
use std::{
    ffi::OsString,
    iter, mem,
    os::windows::prelude::OsStrExt,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, WPARAM},
        windef::{HWND, RECT},
    },
    um::{
        commctrl::InitCommonControls,
        libloaderapi::*,
        wingdi::{CreateFontIndirectW, GetStockObject, DEFAULT_GUI_FONT, FW_BOLD},
        winuser::*,
    },
};

#[cfg(not(target_os = "windows"))]
compile_error!("Windows crash handler can only be compiled for Windows targets");

// TODO: make the crash handler window resizable
// TODO: make the GitHub issue tracker link an actual clickable link
// TODO: make the header ""jokes"" properly get randomized (what a feature!)
// TODO: enable keyboard control of the window (like tab for focus)

// i'm such a comedian
const _HEADERS: &[&'static str] = &[
    "Well this is awkward *laugh track*",
    "here is a wacky randomized error message",
    "something fucked up",
    "Not to worry, we're still flying half a ship",
    "Hello there!",
    "holy fucking bingle. what?! :3",
    "this joke is hidden behind 10 layers of irony",
];

const CONFIG: HandlerConfig = HandlerConfig {
    window_title: "Zenit Crash Handler",

    header_text: "here is a wacky randomized error message",

    // TODO: make the crash report link into a clickable one
    main_text: "Zenit Engine has encountered an unexpected error, causing this\n\
    crash. It's most likely a bug. :(\n\
    \n\
    It may be worth to report it at:\n\
    https://github.com/natanalt/zenit/issues\n\
    \n\
    A detailed crash log is written below.",

    footer_text: "Press Copy to copy this error log to your clipboard, and press OK to exit.",

    copy_text: "Copy",
    ok_text: "OK",

    logo_bitmap_id: 150, // Hardcoded in src/platform/windows/zenit.rc :)
    logo_size: 128,      // Also hardcoded to the bitmap size

    initial_window_width: 500,
    initial_window_height: 500,
    padding: 12,

    header_height: 25,
    footer_height: 25,

    footer_button_width: 100,
    footer_button_height: 30,
};

struct Bounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Bounds {
    pub unsafe fn apply_to(&self, hwnd: HWND) {
        SetWindowPos(hwnd, ptr::null_mut(), self.x, self.y, self.w, self.h, 0);
    }
}

/// Internal configuration, to make everything easier
struct HandlerConfig {
    window_title: &'static str,
    header_text: &'static str,
    main_text: &'static str,
    footer_text: &'static str,
    copy_text: &'static str,
    ok_text: &'static str,

    logo_bitmap_id: i32,
    logo_size: i32,

    initial_window_width: i32,
    initial_window_height: i32,
    padding: i32,

    header_height: i32,
    footer_height: i32,

    footer_button_width: i32,
    footer_button_height: i32,
}

impl HandlerConfig {
    pub const fn compute_layout(&self, full_width: i32, full_height: i32) -> ComputedLayout {
        //
        // The layout is hardcoded, and basically looks like this:
        // ┌──────────────────────────┐
        // │ Zenit Crash         _  X │
        // ├──────────────────────────┤
        // │ ┌────┐ Header            │
        // │ │ICON│ Main text text aa │
        // │ │HERE│ Main text text aa │
        // │ └────┘                   │
        // │ ┌──────────────────────┐ │
        // │ │ Crash logs :(        │ │
        // │ │                      │ │
        // │ │                      │ │
        // │ └──────────────────────┘ │
        // │ Footer text is here      │
        // │                          │
        // │              [Copy] [OK] │
        // └──────────────────────────┘
        //

        let padding = self.padding;
        let width = full_width - padding * 2;
        let height = full_height - padding * 2;
        let mut y = padding;

        let logo = Bounds {
            x: padding,
            y,
            w: self.logo_size,
            h: self.logo_size,
        };

        y += padding / 2;
        let header = Bounds {
            x: padding * 2 + self.logo_size,
            y,
            w: width - logo.w - padding * 2,
            h: self.header_height,
        };

        y += header.h;
        let main = Bounds {
            x: header.x,
            y,
            w: header.w,
            h: logo.h - header.h,
        };

        y = logo.h + padding * 2;
        let logs = Bounds {
            x: padding,
            y,
            w: width,
            h: height
                - (logo.h + padding + self.footer_height + padding + self.footer_button_height),
        };

        y += logs.h + padding;
        let footer = Bounds {
            x: padding,
            y,
            w: width,
            h: self.footer_height,
        };

        y += footer.h;
        let ok = Bounds {
            x: full_width - padding - self.footer_button_width,
            y,
            w: self.footer_button_width,
            h: self.footer_button_height,
        };

        let copy = Bounds {
            x: ok.x - padding - self.footer_button_width,
            y,
            w: self.footer_button_width,
            h: self.footer_button_height,
        };

        ComputedLayout {
            logo,
            header,
            main,
            logs,
            footer,
            copy,
            ok,
        }
    }
}

struct ComputedLayout {
    logo: Bounds,
    header: Bounds,
    main: Bounds,
    logs: Bounds,
    footer: Bounds,
    copy: Bounds,
    ok: Bounds,
}

const BUTTON_OK: i32 = 151;
const BUTTON_COPY: i32 = 152;

macro_rules! verify_window_creation {
    ($hwnd:expr) => {{
        let hwnd = $hwnd;
        if hwnd.is_null() {
            window_create_error();
            return;
        }
        hwnd
    }};
}

/// Set to `true` in the handler's proc function, read by its message loop
static SHOULD_COPY_ERROR_LOG: AtomicBool = AtomicBool::new(false);

pub fn set_panic_hook() {
    // Safety: maybe
    unsafe {
        InitCommonControls();

        let window_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(ptr::null()),
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: COLOR_WINDOW as _,
            lpszMenuName: ptr::null(),
            lpszClassName: wstring("ZenitCrashHandler"),
            hIconSm: ptr::null_mut(),
        };

        if RegisterClassExW(&window_class as _) == 0 {
            const CAPTION: &str = "Couldn't register the crash handler.\n\n\
                Zenit will start anyway, but in case of a crash, Rust's \
                default handler will be used (output to console, if one exists).";
            MessageBoxW(
                ptr::null_mut(),
                wstring(CAPTION),
                wstring("Zenit Engine"),
                MB_OK | MB_ICONWARNING,
            );
            return;
        }
    }

    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| unsafe {
        let hinst = GetModuleHandleW(ptr::null());
        let class_name = wstring("ZenitCrashHandler");
        let static_name = wstring("STATIC");
        let edit_name = wstring("EDIT");
        let button_name = wstring("BUTTON");
        let window_title = wstring(CONFIG.window_title);
        let header_text = wstring(CONFIG.header_text);
        let main_text = wstring(CONFIG.main_text);
        let footer_text = wstring(CONFIG.footer_text);
        let copy_text = wstring(CONFIG.copy_text);
        let ok_text = wstring(CONFIG.ok_text);

        let error_log = wstring(
            super::generate_error_log(info)
                .replace("\n", "\r\n")
                .as_str(),
        );

        let header_font = {
            let mut metrics = mem::zeroed::<NONCLIENTMETRICSW>();
            metrics.cbSize = mem::size_of::<NONCLIENTMETRICSW>() as u32;
            SystemParametersInfoW(
                SPI_GETNONCLIENTMETRICS,
                metrics.cbSize,
                &mut metrics as *mut NONCLIENTMETRICSW as _,
                0,
            );

            let mut font = metrics.lfMessageFont.clone();
            font.lfWeight = FW_BOLD;
            //font.lfItalic = 1;
            CreateFontIndirectW(&font as _)
        };

        let window = verify_window_creation!(CreateWindowExW(
            0,
            class_name,
            window_title,
            WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CONFIG.initial_window_width,
            CONFIG.initial_window_height,
            ptr::null_mut(),
            ptr::null_mut(),
            hinst,
            ptr::null_mut()
        ));

        let new_child_window = |class, caption, style| {
            CreateWindowExW(
                0,
                class,
                caption,
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window,
                ptr::null_mut(),
                hinst,
                ptr::null_mut(),
            )
        };

        let new_child_window_menu = |class, caption, style, menu| {
            CreateWindowExW(
                0,
                class,
                caption,
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window,
                menu as _,
                hinst,
                ptr::null_mut(),
            )
        };

        let logo = verify_window_creation!(new_child_window(
            static_name,
            wstring(""),
            WS_CHILD | WS_VISIBLE | SS_NOTIFY | SS_BITMAP | WS_BORDER,
        ));

        let header = verify_window_creation!(new_child_window(
            static_name,
            header_text,
            WS_CHILD | WS_VISIBLE | SS_NOTIFY,
        ));

        let main = verify_window_creation!(new_child_window(
            static_name,
            main_text,
            WS_CHILD | WS_VISIBLE | SS_NOTIFY,
        ));

        let logs = verify_window_creation!(new_child_window(
            edit_name,
            error_log,
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | ES_MULTILINE
                | ES_AUTOVSCROLL
                | WS_VSCROLL
                | ES_READONLY,
        ));

        let footer = verify_window_creation!(new_child_window(
            static_name,
            footer_text,
            WS_CHILD | WS_VISIBLE | SS_NOTIFY,
        ));

        let button_copy = verify_window_creation!(new_child_window_menu(
            button_name,
            copy_text,
            WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON | BS_TEXT,
            BUTTON_COPY,
        ));

        let button_ok = verify_window_creation!(new_child_window_menu(
            button_name,
            ok_text,
            WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON | BS_TEXT,
            BUTTON_OK,
        ));

        let reset_layout = || {
            let mut rect = mem::zeroed::<RECT>();
            GetClientRect(window, &mut rect as _);

            let layout = CONFIG.compute_layout(rect.right - 1, rect.bottom - 1);
            layout.logo.apply_to(logo);
            layout.header.apply_to(header);
            layout.main.apply_to(main);
            layout.logs.apply_to(logs);
            layout.footer.apply_to(footer);
            layout.copy.apply_to(button_copy);
            layout.ok.apply_to(button_ok);
        };
        reset_layout();

        // Set the logo
        SendMessageW(
            logo,
            STM_SETIMAGE,
            IMAGE_BITMAP as _,
            LoadImageW(
                hinst,
                CONFIG.logo_bitmap_id as _,
                IMAGE_BITMAP,
                128 as _,
                128 as _,
                0,
            ) as _,
        );

        // Update fonts
        EnumChildWindows(
            window,
            Some(set_font),
            GetStockObject(DEFAULT_GUI_FONT as _) as _,
        );
        SendMessageW(header, WM_SETFONT, header_font as _, 1);
        //SendMessageW(logs, WM_SETFONT, GetStockObject(ANSI_FIXED_FONT as _) as _, 1);

        let mut msg = mem::zeroed::<MSG>();
        ShowWindow(window, SW_NORMAL);
        while GetMessageW(&mut msg as _, window, 0, 0) > 0 {
            TranslateMessage(&msg as _);
            DispatchMessageW(&msg as _);

            if SHOULD_COPY_ERROR_LOG.load(Ordering::SeqCst) {
                OpenClipboard(window);
                EmptyClipboard();
                SetClipboardData(CF_UNICODETEXT, error_log as _);
                CloseClipboard();

                SHOULD_COPY_ERROR_LOG.store(false, Ordering::SeqCst);
            }
        }

        old_hook(info);
        std::process::abort();
    }));

    debug!("Panic handler has just been set");
}

unsafe extern "system" fn set_font(hwnd: HWND, font: LPARAM) -> i32 {
    SendMessageW(hwnd, WM_SETFONT, font as _, 1);
    1
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND if wparam == BUTTON_COPY as _ => {
            SHOULD_COPY_ERROR_LOG.store(true, Ordering::SeqCst);
            1
        }
        WM_COMMAND if wparam == BUTTON_OK as _ => {
            DestroyWindow(hwnd);
            1
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn window_create_error() {
    MessageBoxW(
        ptr::null_mut(),
        wstring(
            "Zenit has crashed, but the error handler couldn't be opened for some reason.\n\n\
            That's a lot of errors at once! Zenit will now shutdown, you get no logs back. :(",
        ),
        wstring("Zenit Crash Handler"),
        MB_OK | MB_ICONERROR,
    );
}

/// Converts a UTF-8 string to UTF-16, and returns its pointer.
/// This causes a memory leak, but ✨ I don't care ✨
unsafe fn wstring(source: &str) -> *mut u16 {
    OsString::from(source)
        .encode_wide()
        .chain(iter::once(0u16)) // don't forget the null u16, C is annoying
        .collect::<Vec<u16>>()
        .leak()
        .as_mut_ptr()
}

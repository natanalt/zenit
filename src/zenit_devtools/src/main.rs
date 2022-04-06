use eframe::{NativeOptions, epaint::vec2};

mod app;

fn main() {
    eframe::run_native(
        Box::new(app::DevTools::default()),
        NativeOptions {
            initial_window_size: vec2(1024.0, 600.0).into(),
            min_window_size: vec2(300.0, 300.0).into(),
            resizable: true,
            ..Default::default()
        },
    )
}

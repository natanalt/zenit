use egui::*;
use viewers::Viewer;

mod viewers;

struct ZenitViewer {
    status_bar: String,
    viewers: Vec<Box<dyn Viewer>>,
    current_viewer: usize,
}

impl ZenitViewer {}

impl Default for ZenitViewer {
    fn default() -> Self {
        Self {
            status_bar: "💖 Welcome to the data browser!".to_string(),
            viewers: viewers::default_viewers(),
            current_viewer: 0,
        }
    }
}

impl eframe::App for ZenitViewer {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("TabBrowser")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::widgets::global_dark_light_mode_switch(ui);
                    ui.separator();

                    ui.label("✨ Zenit Viewer");
                    ui.separator();

                    for (i, viewer) in self.viewers.iter().enumerate() {
                        let response =
                            ui.selectable_label(i == self.current_viewer, viewer.get_name());
                        if response.clicked() {
                            self.current_viewer = i;
                        }
                    }
                });
            });
        
        TopBottomPanel::bottom("StatusBar").show(ctx, |ui| {
            ui.horizontal(|ui| ui.label(&self.status_bar));
        });

        self.viewers[self.current_viewer].update(ctx, frame, &mut self.status_bar);
    }
}

fn main() {
    eframe::run_native(
        "Zenit Data Viewer",
        eframe::NativeOptions {
            initial_window_size: Some(vec2(800.0, 600.0)),
            resizable: true,
            ..Default::default()
        },
        Box::new(|cc| {
            Box::new(ZenitViewer::default())
        }),
    );
}

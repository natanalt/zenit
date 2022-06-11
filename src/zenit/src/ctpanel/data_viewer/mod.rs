use super::{CtResponse, CtWidget};
use crate::engine::{Engine, FrameInfo};

pub struct DataViewer {
    pub visible: bool,
}

impl Default for DataViewer {
    fn default() -> Self {
        Self {
            visible: true,
        }
    }
}

impl CtWidget for DataViewer {
    fn show(&mut self, ctx: &egui::Context, _: &FrameInfo, _engine: &mut Engine) -> CtResponse {
        egui::Window::new("Game Data Viewer")
            .resizable(true)
            .default_size([300.0, 300.0])
            .open(&mut self.visible)
            .show(ctx, |ui| {
                // Hack to make resizing work properly
                let _ = ui.allocate_space(egui::vec2(ui.available_size().x, 0.0));

                egui::SidePanel::left("data_viewer_side").show_inside(ui, |ui| {
                    ui.label(":flushed:");
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.label("gridlock gridlock gridlock gridlock gridlock girlcock gridlock gridlock ");
                });
            });

        CtResponse::default()
    }

    fn should_be_retained(&self) -> bool {
        self.visible
    }
}

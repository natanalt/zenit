use super::{CtResponse, CtWidget, ext::EguiUiExtensions};
use crate::engine::{Engine, FrameInfo};

pub struct TextureViewerWindow {
    pub visible: bool,
    render_state: Option<RenderState>,
}

impl Default for TextureViewerWindow {
    fn default() -> Self {
        Self { visible: true, render_state: None, }
    }
}

struct RenderState {
    
}

impl CtWidget for TextureViewerWindow {
    fn show(
        &mut self,
        ctx: &egui::Context,
        _frame: &FrameInfo,
        _engine: &mut Engine,
    ) -> CtResponse {
        let response = CtResponse::default();

        if self.render_state.is_none() {

        }

        egui::Window::new("Texture Viewer")
            .open(&mut self.visible)
            .default_size([500.0, 400.0])
            .show(ctx, |ui| {
                egui::SidePanel::left("texture_viewer_side").show_inside(ui, |ui| {
                    ui.label("nya");
                });

                ui.horizontal(|ui| {
                    if ui.button("Close file").clicked() {
                        // ...
                    }
    
                    ui.e_faint_label("side/all.lvl");
                });
                ui.separator();

                egui::Frame::dark_canvas(ui.style())
                    .inner_margin(1.0)
                    .show(ui, |ui| {
                        ui.allocate_exact_size(ui.available_size() / egui::vec2(1.0, 1.5), egui::Sense::click_and_drag());
                        ui.label("uwu");
                    });

            });

        response
    }

    fn should_be_retained(&self) -> bool {
        self.visible
    }
}

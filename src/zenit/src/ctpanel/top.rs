use super::{ext::EguiUiExtensions, CtWidget, CtResponse};
use crate::engine::{Engine, FrameInfo};
use zenit_utils::default_fn as default;

pub struct TopView;

impl CtWidget for TopView {
    fn show(&mut self, ctx: &egui::Context, _: &FrameInfo, _: &mut Engine) -> CtResponse {
        egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(format!("🚀 Zenit Engine {}", crate::VERSION))
                    .on_hover_ui(|ui| {
                        egui::Grid::new("build_data").show(ui, |ui| {
                            if cfg!(debug_assertions) {
                                ui.e_faint_label("Debug build");
                            } else {
                                ui.e_faint_label("Release build");
                            }
                            ui.end_row();

                            ui.label("Commit:");
                            ui.e_faint_label(&env!("VERGEN_GIT_SHA")[0..7]);
                            ui.end_row();

                            ui.label("Built on:");
                            ui.e_faint_label(env!("VERGEN_BUILD_DATE"));
                            ui.end_row();

                            ui.label("Rustc:");
                            ui.e_faint_label(env!("VERGEN_RUSTC_SEMVER"));
                            ui.end_row();
                        });

                        // Explicitly set the width, as separator takes up all
                        // available space by default
                        ui.allocate_ui(egui::vec2(150.0, 6.0), egui::Ui::separator);

                        ui.colored_label(egui::Color32::RED, "♥");
                    });

                ui.separator();

                ui.label("Tools:");
                let _ = ui.button("Game Data Viewer");
                let _ = ui.button("Log Viewer");
                let _ = ui.button("Resource Tracker");
            });
        });

        default()
    }
}

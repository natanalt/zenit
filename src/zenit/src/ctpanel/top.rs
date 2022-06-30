use super::{ext::EguiUiExtensions, texture_viewer::{TextureViewerWindow, TextureViewerBundle}};
use crate::schedule::TopFrameStage;
use bevy_ecs::prelude::*;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(
        TopFrameStage::ControlPanel,
        top_view.after(super::side::side_view),
    );
}

pub fn top_view(
    mut commands: Commands,
    texture_viewers: Query<&TextureViewerWindow>,
    ctx: ResMut<egui::Context>,
) {
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

            ui.menu_button("Resource Viewers...", |ui| {
                let texture_viewer_button = ui.add_enabled(
                    texture_viewers.is_empty(),
                    egui::Button::new("Texture Viewer"),
                );

                if texture_viewer_button.clicked() {
                    commands.spawn_bundle(TextureViewerBundle::default());
                    ui.close_menu();
                }
            });

            //if ui.button("Game Data Viewer").clicked() {
            //    new_widgets.push(Box::new(DataViewerWindow::default()) as _);
            //}

            let _ = ui.button("Log Viewer");
            let _ = ui.button("Resource Tracker");
        });
    });
}

use std::time::Duration;

use super::ext::EguiUiExtensions;
use crate::{profiling::FrameProfiler, render::base::RenderContext, schedule::TopFrameStage};
use bevy_ecs::prelude::*;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::ControlPanel, side_view);
}

pub fn side_view(
    mut profiler: ResMut<FrameProfiler>,
    context: Res<RenderContext>,
    ctx: ResMut<egui::Context>,
) {
    egui::SidePanel::left("left_panel")
        .default_width(250.0)
        .resizable(true)
        .show(&ctx, |ui| {
            egui::Grid::new("device_info").show(ui, |ui| {
                let info = &context.adapter_info;

                ui.label("Device:");
                ui.e_faint_label(&info.name).on_hover_ui(|ui| {
                    egui::Grid::new("device_details").show(ui, |ui| {
                        ui.label("Device type:");
                        ui.e_faint_label(format!("{:?}", info.device_type));
                        ui.end_row();

                        ui.label("PCI vendor:");
                        ui.e_faint_label(format!("0x{:04X}", info.vendor));
                        ui.end_row();

                        ui.label("PCI device:");
                        ui.e_faint_label(format!("0x{:04X}", info.device));
                        ui.end_row();
                    });
                });
                ui.end_row();

                ui.label("Backend:");
                ui.e_faint_label(format!("{:?}", &info.backend));
                ui.end_row();
            });
            ui.separator();

            egui::CollapsingHeader::new("Frame times")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("framerate_grid").show(ui, |ui| {
                        ui.label("Framerate");
                        ui.e_faint_label(format!("{} FPS", profiler.fps));
                        ui.end_row();

                        let avg_frame_time = Duration::from_secs_f32(1.0 / profiler.fps as f32);
                        ui.label("Frame time");
                        ui.e_faint_label(format!("{:?}", avg_frame_time))
                            .on_hover_text("(Average from the last second)");
                        ui.end_row();
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("Frame graph");
                        ui.e_faint_label(format!("(Target FPS: {})", profiler.target_fps));
                    });

                    egui::Frame::dark_canvas(&ctx.style()).show(ui, |ui| {
                        ui.allocate_at_least(
                            egui::vec2(ui.available_size().x, 100.0),
                            egui::Sense::click_and_drag(),
                        );
                    });
                });

            ui.add_space(10.0);

            egui::CollapsingHeader::new("Game info")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("A game is selected, TODO UI");
                });
        });
}

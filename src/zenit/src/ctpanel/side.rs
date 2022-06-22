use crate::{render::base::RenderContext, schedule::TopFrameStage, profiling::FrameProfiler};
use super::ext::EguiUiExtensions;
use bevy_ecs::prelude::*;
use std::sync::Mutex;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::ControlPanel, side_view);
}

pub fn side_view(
    mut profiler: ResMut<FrameProfiler>,
    context: Res<RenderContext>,
    ctx: Res<Mutex<egui::Context>>,
) {
    let ctx = ctx.lock().unwrap();

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
                    ui.label(format!("{} FPS", profiler.fps));
                    ui.add_space(5.0);                    
                });

            ui.add_space(10.0);

            egui::CollapsingHeader::new("Game info")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("A game is selected, TODO UI");
                });
        });
}

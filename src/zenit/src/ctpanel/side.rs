use super::{ext::EguiUiExtensions, CtResponse, CtWidget};
use crate::engine::{Engine, FrameInfo};
use std::time::Duration;
use zenit_utils::default_fn as default;

pub struct SideView;

impl CtWidget for SideView {
    fn show(&mut self, ctx: &egui::Context, _frame: &FrameInfo, engine: &mut Engine) -> CtResponse {
        let response = CtResponse::default();

        egui::SidePanel::left("left_panel")
            .default_width(250.0)
            .resizable(true)
            .show(&ctx, |ui| {
                egui::Grid::new("device_info").show(ui, |ui| {
                    let info = &engine.renderer.context.device_info;

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
                        let fill_space_amount = ui.available_size_before_wrap();
                        let (rect, _response) = ui.allocate_at_least(
                            egui::vec2(fill_space_amount.x, 100.0),
                            egui::Sense::hover(),
                        );

                        // Frame time graph prototype
                        // TODO: clean up frame time graph (put it into a struct or smth)
                        {
                            let mut shapes = vec![];

                            shapes.push(egui::Shape::Rect(egui::epaint::RectShape {
                                rect,
                                rounding: ui.style().noninteractive().rounding,
                                fill: ui.visuals().extreme_bg_color,
                                stroke: ui.style().noninteractive().bg_stroke,
                            }));

                            let height = 1.0 / 50.0;
                            let total_samples = rect.width().floor() as usize;
                            let mut base = rect.max;
                            for frame in engine.frame_profiler.frames.iter().take(total_samples) {
                                let mut y = base.y;

                                let ui_time = frame.ui_time.as_secs_f32();
                                let ui_height = (ui_time / height) * rect.height();

                                let game_time = frame.game_time.as_secs_f32();
                                let game_height = (game_time / height) * rect.height();

                                let render_time = frame.render_time.as_secs_f32();
                                let render_height = (render_time / height) * rect.height();

                                shapes.push(egui::Shape::LineSegment {
                                    points: [
                                        egui::pos2(base.x, y),
                                        egui::pos2(base.x, y - render_height),
                                    ],
                                    stroke: egui::Stroke::new(
                                        1.0,
                                        engine.frame_profiler.render_color,
                                    ),
                                });
                                y -= render_height;

                                shapes.push(egui::Shape::LineSegment {
                                    points: [
                                        egui::pos2(base.x, y),
                                        egui::pos2(base.x, y - game_height),
                                    ],
                                    stroke: egui::Stroke::new(
                                        1.0,
                                        engine.frame_profiler.game_color,
                                    ),
                                });
                                y -= game_height;

                                shapes.push(egui::Shape::LineSegment {
                                    points: [
                                        egui::pos2(base.x, y),
                                        egui::pos2(base.x, y - ui_height),
                                    ],
                                    stroke: egui::Stroke::new(1.0, engine.frame_profiler.ui_color),
                                });

                                base.x -= 1.0;
                            }

                            ui.painter().extend(shapes);
                        }

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("FPS:");
                            ui.e_faint_label(engine.frame_profiler.calculate_fps().to_string());
                        });

                        ui.add_space(5.0);
                        ui.label("Legend:");
                        ui.indent("legend_indent", |ui| {
                            ui.horizontal(|ui| {
                                ui.color_edit_button_srgba(&mut engine.frame_profiler.render_color);
                                ui.label("Render phase");
                            });
                            ui.horizontal(|ui| {
                                ui.color_edit_button_srgba(&mut engine.frame_profiler.game_color);
                                ui.label("Game phase");
                            });
                            ui.horizontal(|ui| {
                                ui.color_edit_button_srgba(&mut engine.frame_profiler.ui_color);
                                ui.label("UI phase");
                            });
                        });

                        ui.add_space(5.0);
                        ui.label("Limits:");
                        ui.indent("frame_limits", |ui| {
                            egui::Grid::new("timing_info").show(ui, |ui| {
                                ui.label("Min time:");
                                ui.e_faint_label(format!("{:?}", engine.frame_profiler.min_time));
                                ui.end_row();

                                ui.label("Max time:");
                                ui.e_faint_label(format!("{:?}", engine.frame_profiler.max_time));
                                ui.end_row();

                                ui.label("Avg time:");
                                ui.e_faint_label(format!("{:?}", engine.frame_profiler.avg_time));
                                ui.end_row();
                            })
                        });

                        ui.add_space(5.0);
                        ui.label("Last frame:");
                        ui.indent("last_frame", |ui| {
                            egui::Grid::new("timing_info").show(ui, |ui| {
                                let last = engine
                                    .frame_profiler
                                    .frames
                                    .front()
                                    .map(Clone::clone)
                                    .unwrap_or(default());
                                let total = last.total_time();

                                let percentage = |part: Duration| {
                                    let total_secs = total.as_secs_f32();
                                    let part_secs = part.as_secs_f32();
                                    (part_secs / total_secs) * 100.0
                                };

                                ui.label("Total time:");
                                ui.e_faint_label("(100.00 %)");
                                ui.e_faint_label(format!("{:?}", total));
                                ui.end_row();

                                ui.label("UI time:");
                                ui.e_faint_label(format!("({:.02} %)", percentage(last.ui_time)));
                                ui.e_faint_label(format!("{:?}", last.ui_time));
                                ui.end_row();

                                ui.label("Game time:");
                                ui.e_faint_label(format!("({:.02} %)", percentage(last.game_time)));
                                ui.e_faint_label(format!("{:?}", last.game_time));
                                ui.end_row();

                                ui.label("Render time:");
                                ui.e_faint_label(format!(
                                    "({:.02} %)",
                                    percentage(last.render_time)
                                ));
                                ui.e_faint_label(format!("{:?}", last.render_time));
                                ui.end_row();
                            })
                        });

                        ui.add_space(5.0);
                        if ui.button("Reset counters").clicked() {
                            engine.frame_profiler.reset();
                        }

                        ui.add_space(5.0);
                    });

                ui.add_space(10.0);

                egui::CollapsingHeader::new("Game info")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("A game is selected, TODO UI");
                    });
            });

        response
    }
}

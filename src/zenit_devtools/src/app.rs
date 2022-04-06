use eframe::{egui::*, epi::Frame, epi::*};

#[derive(Default)]
pub struct DevTools {}

impl App for DevTools {
    fn setup(&mut self, ctx: &Context, frame: &Frame, _storage: Option<&dyn Storage>) {}

    fn update(&mut self, ctx: &Context, _frame: &Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("♥ Zenit Tools");
                ui.separator();
            })
        });

        SidePanel::left("left_panel").show(ctx, |ui| {});

        CentralPanel::default().show(ctx, |ui| {});
    }

    fn name(&self) -> &str {
        "Zenit Developer Tools"
    }
}

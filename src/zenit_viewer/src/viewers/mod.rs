
pub trait Viewer {
    fn get_name(&self) -> &str;
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        status_bar: &mut String,
    );
}

struct Blank {
    label: &'static str,
}

impl Viewer for Blank {
    fn get_name(&self) -> &str {
        self.label
    }

    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        status_bar: &mut String,
    ) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label(format!("nya {}", self.label));
            });
    }
}

pub fn default_viewers() -> Vec<Box<dyn Viewer>> {
    vec![
        Box::new(Blank { label: "asdkasjdk" }),
        Box::new(Blank { label: "nya " }),
        Box::new(Blank { label: "Model Viewer" }),
        Box::new(Blank { label: "Level Viewer" }),
    ]
}

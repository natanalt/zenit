use super::{ext::EguiUiExtensions, side::SideView, top::TopView, CtResponse, CtWidget};
use crate::{
    engine::{Engine, FrameInfo},
    root::GameRoot,
};
use std::path::PathBuf;

pub struct RootSelectWindow {
    pub visible: bool,
    pub path_buffer: String,
}

impl Default for RootSelectWindow {
    fn default() -> Self {
        Self {
            visible: true,
            path_buffer: String::new(),
        }
    }
}

impl CtWidget for RootSelectWindow {
    fn show(&mut self, ctx: &egui::Context, _: &FrameInfo, _: &mut Engine) -> CtResponse {
        let mut response = CtResponse::default();

        egui::Window::new("Select Game Root")
            .resizable(false)
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Please select Star Wars Battlefront II root directory.");
                ui.e_faint_label("(The one that contains a GameData directory)");
                ui.add_space(5.0);
                ui.text_edit_singleline(&mut self.path_buffer);
                ui.add_space(5.0);
                ui.e_faint_label(
                    "It can also be specified with the --game-root command line parameter.",
                );
                ui.add_space(5.0);
                if ui.button("OK").clicked() {
                    let new_root = GameRoot::new(Some(PathBuf::from(&self.path_buffer)).as_ref());
                    if new_root.is_invalid() {
                        response.add_error_box("Error", "Invalid root directory");
                    } else {
                        self.visible = false;
                        response.add_widget(SideView);
                        response.add_widget(TopView);
                    }
                }
            });

        response
    }

    fn should_be_retained(&self) -> bool {
        self.visible
    }
}

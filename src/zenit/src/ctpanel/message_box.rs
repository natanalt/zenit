use super::{CtResponse, CtWidget};
use crate::engine::{Engine, FrameInfo};

pub struct MessageBox {
    pub id: egui::Id,
    pub visible: bool,
    pub title: String,
    pub message: String,
}

impl MessageBox {
    // Currently those functions are identical, they will be changed someday
    // TODO: icons in message boxes (and make them prettier in general)

    pub fn info(title: &str, message: &str) -> Self {
        Self {
            id: egui::Id::new(zenit_utils::counter::next()),
            visible: true,
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    pub fn warn(title: &str, message: &str) -> Self {
        Self::info(title, message)
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self::info(title, message)
    }
}

impl CtWidget for MessageBox {
    fn show(&mut self, ctx: &egui::Context, _: &FrameInfo, _: &mut Engine) -> CtResponse {
        let mut ok_pressed = false;
        egui::Window::new(&self.title)
            .id(self.id)
            .resizable(false)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                ui.label(&self.message);
                ui.separator();
                ok_pressed = ui.button("OK").clicked();
            });

        if ok_pressed {
            self.visible = false;
        }

        CtResponse::default()
    }

    fn is_unique(&self) -> bool {
        false
    }

    fn should_be_retained(&self) -> bool {
        self.visible
    }
}

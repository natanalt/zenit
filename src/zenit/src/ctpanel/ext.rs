use bevy_ecs::prelude::*;

use super::Widget;

pub trait EguiUiExtensions {
    /// Works like normal [`egui::Ui::label`], but prints the text in a "weak"
    /// color
    fn e_faint_label(&mut self, text: impl Into<egui::RichText>) -> egui::Response;
    fn e_faint_mono_label(&mut self, text: impl Into<egui::RichText>) -> egui::Response;
}

impl EguiUiExtensions for egui::Ui {
    fn e_faint_label(&mut self, text: impl Into<egui::RichText>) -> egui::Response {
        self.colored_label(self.style().visuals.weak_text_color(), text)
    }

    fn e_faint_mono_label(&mut self, text: impl Into<egui::RichText>) -> egui::Response {
        self.label(
            text.into()
                .monospace()
                .color(self.style().visuals.weak_text_color()),
        )
    }
}

pub trait BevyCommandsExtensions {
    fn e_spawn_with(&mut self, c: impl Component) -> Entity;
    fn e_info_box(&mut self, title: &str, message: &str) -> Entity;
    fn e_warn_box(&mut self, title: &str, message: &str) -> Entity;
    fn e_error_box(&mut self, title: &str, message: &str) -> Entity;
}

impl<'w, 's> BevyCommandsExtensions for Commands<'w, 's> {
    fn e_spawn_with(&mut self, c: impl Component) -> Entity {
        self.spawn().insert(c).id()
    }

    fn e_info_box(&mut self, title: &str, message: &str) -> Entity {
        self.spawn()
            .insert(Widget)
            .insert(super::message_box::MessageBox::info(title, message))
            .id()
    }

    fn e_warn_box(&mut self, title: &str, message: &str) -> Entity {
        self.spawn()
            .insert(Widget)
            .insert(super::message_box::MessageBox::warn(title, message))
            .id()
    }

    fn e_error_box(&mut self, title: &str, message: &str) -> Entity {
        self.spawn()
            .insert(Widget)
            .insert(super::message_box::MessageBox::error(title, message))
            .id()
    }
}

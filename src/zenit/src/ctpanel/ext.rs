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

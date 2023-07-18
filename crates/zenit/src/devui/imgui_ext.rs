use imgui::Ui;


/// Zenit-specific extensions to the imgui-rs API
pub trait UiExt {
    fn bullet_field(&self, header: impl AsRef<str>, field: impl AsRef<str>);
    fn question_tooltip(&self, text: impl AsRef<str>);
}

impl UiExt for Ui {
    fn bullet_field(&self, header: impl AsRef<str>, field: impl AsRef<str>) {
        let _group_token = self.begin_group();
        self.bullet();
        self.text_disabled(format!("{}:", header.as_ref()));
        self.same_line();
        self.text(field);
    }

    fn question_tooltip(&self, text: impl AsRef<str>) {
        self.text_disabled("(?)");
        if self.is_item_hovered() {
            self.tooltip_text(text);
        }
    }
}

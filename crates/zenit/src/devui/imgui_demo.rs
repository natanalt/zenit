use super::WidgetResponse;
use crate::scene::EngineBorrow;
use imgui::Ui;

pub fn imgui_demo(ui: &mut Ui, _: &mut EngineBorrow) -> WidgetResponse {
    let mut open = true;
    ui.show_demo_window(&mut open);
    WidgetResponse::keep_opened(open)
}

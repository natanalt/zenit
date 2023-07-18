use super::{DevUiComponent, DevUiTextures};
use crate::{entities::{Universe, Component}, scene::EngineBorrow};
use imgui::Ui;

pub fn add_imgui_demo(uni: &mut Universe) {
    if uni.get_components::<ImGuiDemo>().next().is_some() {
        return;
    }

    uni.build_entity()
        .with_component(ImGuiDemo)
        .with_component(DevUiComponent {
            widget: Some(Box::new(imgui_demo)),
        })
        .finish();
}

struct ImGuiDemo;
impl Component for ImGuiDemo {}

fn imgui_demo(ui: &mut Ui, _: &mut EngineBorrow, _: &mut DevUiTextures) -> bool {
    let mut open = true;
    ui.show_demo_window(&mut open);
    open
}

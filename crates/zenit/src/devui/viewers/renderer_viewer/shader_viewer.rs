use crate::scene::EngineBorrow;
use imgui::Ui;

#[derive(Default)]
pub struct ShaderViewer {
    current: Option<(String, String)>,
}

impl super::RendererViewerTab for ShaderViewer {
    fn show_ui(&mut self, ui: &Ui, engine: &mut EngineBorrow) {
        ui.child_window("ShaderList")
            .size([150.0, 0.0])
            .border(true)
            .horizontal_scrollbar(true)
            .build(|| {
                for (name, shader) in &engine.renderer.shaders {
                    let is_selected = match &self.current {
                        Some((other_name, _)) if name == other_name => true,
                        _ => false,
                    };

                    if ui.selectable_config(name).selected(is_selected).build() {
                        self.current = Some((name.clone(), shader.code.clone()));
                    }
                }
            });

        ui.same_line();

        ui.child_window("ShaderViewer")
            .horizontal_scrollbar(true)
            .size([0.0, 0.0])
            .build(|| match &mut self.current {
                Some((_, code)) => {
                    ui.input_text_multiline("##0", code, ui.content_region_avail())
                        .read_only(true)
                        .build();
                }
                None => {
                    ui.text("No shader selected.");
                }
            });
    }
}

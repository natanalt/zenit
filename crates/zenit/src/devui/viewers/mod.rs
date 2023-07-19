use super::{imgui_demo::imgui_demo, DevUiWidget};

mod model_preview;
mod renderer_viewer;

pub const TOOLS: &[(&str, fn() -> Box<dyn DevUiWidget>)] = &[
    ("File Explorer", || todo!("todo: start the file explorer")),
    ("Renderer Tools", || {
        Box::new(renderer_viewer::RendererViewer::default())
    }),
    ("Dear ImGui Demo", || Box::new(imgui_demo)),
];

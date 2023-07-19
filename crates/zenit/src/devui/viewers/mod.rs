use super::{DevUiWidget, imgui_demo::imgui_demo};

mod renderer_viewer;
mod model_preview;

pub const TOOLS: &[(&str, fn() -> Box<dyn DevUiWidget>)] = &[
    ("File Explorer", || todo!("todo: start the file explorer")),
    ("Renderer Tools", || Box::new(renderer_viewer::RendererViewer::default())),
    ("Dear ImGui Demo", || Box::new(imgui_demo)),
];

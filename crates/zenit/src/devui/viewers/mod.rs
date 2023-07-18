use crate::entities::Universe;

mod renderer_viewer;

pub const TOOLS: &[(&str, fn(&mut Universe))] = &[
    ("File Explorer", |_uni| println!("todo: start the file explorer")),
    ("Renderer Tools", |uni| renderer_viewer::RendererViewer::add(uni)),
    ("ImGui Demo", |uni| super::imgui_demo::add_imgui_demo(uni)),
];

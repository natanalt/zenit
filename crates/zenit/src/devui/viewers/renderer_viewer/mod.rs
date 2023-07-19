use crate::{
    devui::{imgui_ext::UiExt, DevUiWidget, WidgetResponse},
    scene::EngineBorrow,
};
use glam::*;
use imgui::{TabBar, TabItem, Ui};
use wgpu::AdapterInfo;

mod shader_viewer;
use shader_viewer::*;
mod texture_viewer;
use texture_viewer::*;

pub struct RendererViewer {
    id: u64,
    tabs: Vec<(&'static str, Box<dyn RendererViewerTab>)>,
}

impl Default for RendererViewer {
    fn default() -> Self {
        Self {
            id: zenit_utils::counter::next(),
            tabs: vec![
                ("Textures", Box::new(TextureViewer::default()) as _),
                ("Shaders", Box::new(ShaderViewer::default()) as _),
            ],
        }
    }
}

impl DevUiWidget for RendererViewer {
    fn process_ui(&mut self, ui: &mut Ui, engine: &mut EngineBorrow) -> WidgetResponse {
        let mut opened = true;
        let Some(_window_token) = ui
            .window(format!("Renderer Tools##{}", self.id))
            .opened(&mut opened)
            .size([650.0, 500.0], imgui::Condition::Appearing)
            .begin()
        else { return WidgetResponse::CLOSED };

        let Some(_tabs_token) = TabBar::new("RendererTabs").reorderable(true).begin(ui)
        else {
            return WidgetResponse::keep_opened(opened);
        };

        let renderer = &mut engine.renderer;
        TabItem::new("General").build(ui, || {
            ui.group(|| {
                ui.text_disabled("Basic information");
                ui.separator();

                let AdapterInfo {
                    name,
                    vendor,
                    device,
                    device_type,
                    driver,
                    driver_info,
                    backend,
                } = renderer.dc.adapter.get_info();

                ui.bullet_field("Adapter", format!("{}", name));
                ui.same_line();
                ui.text_disabled("(?)");
                if ui.is_item_hovered() {
                    let _tooltip_token = ui.begin_tooltip();

                    ui.bullet_field("Vendor/Device", format!("{:04X}/{:04X}", vendor, device));
                    ui.bullet_field("Device type", format!("{:?}", device_type));
                    ui.bullet_field("Driver", format!("{} ({})", driver, driver_info));
                }

                ui.bullet_field("Backend", format!("{:?}", backend));
            });

            ui.spacing();

            #[rustfmt::skip]
            ui.group(|| {
                ui.text_disabled("Resource statistics");
                ui.separator();

                ui.bullet_field("Live textures", format!("{}", renderer.textures.count_elements()));
                ui.bullet_field("Live cubemaps", format!("{}", renderer.cubemaps.count_elements()));
                ui.bullet_field("Live cameras", format!("{}", renderer.cameras.count_elements()));
                ui.bullet_field("Live skyboxes", format!("{}", renderer.skyboxes.count_elements()));
                ui.bullet_field("Loaded shaders", format!("{}", renderer.shaders.len()));

                // lol
                if ui.button("Print wgpu memory report to console") {
                    eprintln!();
                    eprintln!("{:#?}", renderer.dc.instance.generate_report());
                    eprintln!();
                }
            });
        });

        for (name, viewer) in &mut self.tabs {
            TabItem::new(name).build(ui, || viewer.show_ui(ui, engine));
        }

        if !opened {
            for (name, mut viewer) in self.tabs.drain(..) {
                let _ = name;
                viewer.destroy(engine);
            }
        }

        WidgetResponse::keep_opened(opened)
    }
}

trait RendererViewerTab: Send + Sync {
    fn show_ui(&mut self, ui: &Ui, engine: &mut EngineBorrow);
    fn destroy(&mut self, engine: &mut EngineBorrow) {
        let _ = engine;
    }
}

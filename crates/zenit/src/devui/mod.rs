//! The developer's UI implementation, also just called DevUi

use std::{mem, sync::Arc};

use crate::{entities::Component, graphics::system::EguiOutput, scene::EngineBorrow};
use egui::{CentralPanel, TopBottomPanel};
use winit::{event::WindowEvent, window::Window};

pub struct DevUi {
    pub context: egui::Context,
    pub winit_integration: egui_winit::State,
}

impl DevUi {
    pub fn new(window: &Window) -> Self {
        Self {
            context: egui::Context::default(),
            winit_integration: egui_winit::State::new(window),
        }
    }

    pub fn process(&mut self, engine: &mut EngineBorrow) {
        let EngineBorrow {
            globals: gs,
            assets: _,
            renderer,
            universe,
        } = engine;

        for event in gs.new_messages_of::<WindowEvent<'static>>() {
            let _ = self.winit_integration.on_event(&self.context, event);
        }

        let mut output = self.context.run(
            self.winit_integration
                .take_egui_input(&gs.get::<Arc<Window>>()),
            |ctx| {
                TopBottomPanel::top("GlobalTopBar").show(ctx, |ui| {
                    ui.menu_button("Debug Viewers", |ui| if ui.button("").clicked() {})
                });

                CentralPanel::default().show(ctx, |ui| {
                    for (_entity, ui_component) in universe.get_components_mut::<DevUiComponent>() {
                        ui_component.callback.process_ui(ui);
                    }
                });
            },
        );

        *renderer.egui_output.lock() = EguiOutput {
            tesselated: self.context.tessellate(mem::take(&mut output.shapes)),
            full_output: output,
        };
    }
}

pub struct DevUiComponent {
    pub callback: Box<dyn DevUiWidget>,
}

impl Component for DevUiComponent {}

pub trait DevUiWidget: Send + Sync {
    fn process_ui(&mut self, ui: &egui::Ui);
}

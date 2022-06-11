//! Control panel is the main engine interface, hosting the actual game within
//! a tiny window. Call it a developer UI if you want to.
use self::{side::SideView, top::TopView};
use crate::{
    engine::{Engine, FrameInfo},
    render::{
        base::{screen::Screen, target::RenderTarget},
        layers::EguiLayer,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use winit::{event_loop::EventLoop, window::Window};
use zenit_utils::default_fn as default;

pub mod ext;
pub mod side;
pub mod top;

pub struct EguiManager {
    pub context: egui::Context,
    pub winit_state: RwLock<egui_winit::State>,
    pub last_output: RwLock<egui::FullOutput>,

    /// Used by the egui RenderLayer
    pub pixels_per_point: RwLock<f32>,
}

impl EguiManager {
    pub fn new<T>(_device: &wgpu::Device, el: &EventLoop<T>) -> Self {
        Self {
            context: default(),
            winit_state: RwLock::new(egui_winit::State::new(el)),
            last_output: default(),
            pixels_per_point: RwLock::new(1.0),
        }
    }

    /// Runs the manager and stores the outputs in `self.last_output`,
    /// to be used by the rendering code.
    pub fn run(&self, window: &Window, run_ui: impl FnOnce(&egui::Context)) {
        *self.last_output.write().unwrap() = self.context.run(
            self.winit_state.write().unwrap().take_egui_input(window),
            run_ui,
        );
    }
}

pub struct ControlPanel {
    pub egui_manager: Arc<EguiManager>,
    pub widgets: HashMap<String, Box<dyn CtWidget>>,
}

impl ControlPanel {
    pub fn new<T>(device: &wgpu::Device, el: &EventLoop<T>) -> Self {
        Self {
            egui_manager: Arc::new(EguiManager::new(device, el)),
            widgets: HashMap::default(),
        }
    }

    pub fn frame(&mut self, info: &FrameInfo, engine: &mut Engine) {
        if info.is_on_first_frame() {
            let renderer = &mut engine.renderer;
            let egui_layer = EguiLayer::new(
                &renderer.context.device,
                self.egui_manager.clone(),
                renderer.main_window.get_format(),
            );

            self.insert_widget("Side View", SideView);
            self.insert_widget("Top View", TopView);

            renderer.screens.push(Screen {
                label: Some("Control panel".into()),
                target: renderer.main_window.clone(),
                layers: vec![Arc::new(egui_layer)],
            })
        }

        *self.egui_manager.pixels_per_point.write().unwrap() = engine.window.scale_factor() as _;

        let window = engine.window.clone();
        self.egui_manager.run(&window, |ctx| {
            for (_, widget) in &mut self.widgets {
                widget.show(ctx, info, engine);
            }
        });

        self.widgets.retain(|_, w| !w.should_be_removed());
    }

    pub fn insert_widget(&mut self, name: &str, widget: impl CtWidget + 'static) {
        self.widgets.insert(name.to_string(), Box::new(widget));
    }
}

#[derive(Default)]
pub struct CtResponse {
    pub new_widgets: HashMap<String, Box<dyn CtWidget>>,
}

pub trait CtWidget {
    fn show(&mut self, ctx: &egui::Context, frame: &FrameInfo, engine: &mut Engine) -> CtResponse;

    fn should_be_removed(&self) -> bool {
        false
    }
}

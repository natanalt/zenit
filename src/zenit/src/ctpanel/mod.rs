//! Control panel is the main engine interface, hosting the actual game within
//! a tiny window. Call it a developer UI if you want to.
use self::{side::SideView, top::TopView, root_select::RootSelectWindow, message_box::MessageBox};
use crate::{
    engine::{Engine, FrameInfo},
    render::{
        base::{screen::Screen, target::RenderTarget},
        layers::EguiLayer,
    },
};
use std::{sync::{Arc, RwLock}, any::Any};
use winit::{event_loop::EventLoop, window::Window};
use zenit_utils::default_fn as default;

pub mod ext;
pub mod side;
pub mod top;
pub mod data_viewer;
pub mod root_select;
pub mod message_box;

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
    pub widgets: Vec<Box<dyn CtWidget>>,
}

impl ControlPanel {
    pub fn new<T>(device: &wgpu::Device, el: &EventLoop<T>) -> Self {
        Self {
            egui_manager: Arc::new(EguiManager::new(device, el)),
            widgets: vec![],
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

            if !engine.game_root.is_invalid() {
                self.widgets.push(Box::new(SideView));
                self.widgets.push(Box::new(TopView));
            } else {
                self.widgets.push(Box::new(RootSelectWindow::default()));
            }

            renderer.screens.push(Screen {
                label: Some("Control panel".into()),
                target: renderer.main_window.clone(),
                layers: vec![Arc::new(egui_layer)],
            })
        }

        *self.egui_manager.pixels_per_point.write().unwrap() = engine.window.scale_factor() as _;

        let mut new_widgets = vec![];

        let window = engine.window.clone();
        self.egui_manager.run(&window, |ctx| {
            for widget in &mut self.widgets {
                let mut response = widget.show(ctx, info, engine);
                new_widgets.append(&mut response.new_widgets);
            }
        });

        self.widgets.retain(|w| w.should_be_retained());

        'top: for new_widget in new_widgets {
            if new_widget.is_unique() {
                for old_widget in &self.widgets {
                    if old_widget.as_ref().type_id() == new_widget.as_ref().type_id() {
                        continue 'top;
                    }
                }
            }
            self.widgets.push(new_widget);
        }
    }
}

#[derive(Default)]
pub struct CtResponse {
    pub new_widgets: Vec<Box<dyn CtWidget>>,
}

impl CtResponse {
    pub fn add_info_box(&mut self, title: &str, message: &str) {
        self.new_widgets.push(Box::new(MessageBox::info(title, message)));
    }

    pub fn add_warn_box(&mut self, title: &str, message: &str) {
        self.new_widgets.push(Box::new(MessageBox::warn(title, message)));
    }

    pub fn add_error_box(&mut self, title: &str, message: &str) {
        self.new_widgets.push(Box::new(MessageBox::error(title, message)));
    }

    pub fn add_widget(&mut self, widget: impl CtWidget) {
        self.new_widgets.push(Box::new(widget));
    }
}

pub trait CtWidget: Any {
    fn show(&mut self, ctx: &egui::Context, frame: &FrameInfo, engine: &mut Engine) -> CtResponse;

    fn is_unique(&self) -> bool {
        true
    }

    fn should_be_retained(&self) -> bool {
        true
    }
}

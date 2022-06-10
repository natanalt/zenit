//! Control panel is the main engine interface, hosting the actual game within
//! a tiny window. Call it a developer UI if you want to.
use crate::{
    engine::{Engine, FrameInfo},
    render::{
        base::{screen::Screen, target::RenderTarget},
        layers::EguiLayer,
    },
};
use std::sync::{Arc, RwLock};
use winit::{event_loop::EventLoop, window::Window};
use zenit_utils::default;

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
    pub game_texture: egui::TextureId,
}

impl ControlPanel {
    pub fn new<T>(device: &wgpu::Device, el: &EventLoop<T>) -> Self {
        Self {
            egui_manager: Arc::new(EguiManager::new(device, el)),

            // Invalid value, updated on the first frame
            game_texture: egui::TextureId::User(0),
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

            let primary_screen = renderer
                .find_screen("Game output")
                .expect("game output texture not found")
                .target
                .clone();

            self.game_texture = egui_layer.rpass.lock().register_native_texture(
                &renderer.context.device,
                &primary_screen.get_current_view(),
                wgpu::FilterMode::Nearest,
            );

            renderer.screens.push(Screen {
                label: Some("Control panel".into()),
                target: renderer.main_window.clone(),
                layers: vec![Arc::new(egui_layer)],
            })
        }

        *self.egui_manager.pixels_per_point.write().unwrap() = engine.window.scale_factor() as _;

        let window = engine.window.clone();
        self.egui_manager.run(&window, |ctx| {
            top::show(info, engine, ctx);
            side::show(info, engine, ctx);

            egui::Window::new("Game Window")
                .fixed_size([800.0, 600.0])
                .show(&ctx, |ui| {
                    //ui.centered_and_justified(|ui| {
                    //    ui.label(
                    //        "The game isn't running at the moment.\n\
                    //        That's weird.",
                    //    );
                    //})

                    egui::Frame::dark_canvas(ui.style())
                        .show(ui, |ui| ui.image(self.game_texture, ui.available_size()));
                });
        });
    }
}

//! Control panel is the main engine interface, hosting the actual game within
//! a tiny window. Call it a developer UI if you want to.
use self::{message_box::MessageBox, root_select::RootSelectWindow, side::SideView, top::TopView};
use crate::{
    engine::{Engine, FrameInfo},
    render::base::RenderContext,
};
use egui_wgpu::renderer::RenderPass as EguiRenderPass;
use std::any::Any;
use winit::event_loop::EventLoop;
pub mod ext;
pub mod message_box;
pub mod root_select;
pub mod side;
pub mod top;
pub mod texture_viewer;

pub struct ControlPanel {
    pub egui_context: egui::Context,
    pub egui_winit_state: egui_winit::State,
    pub egui_pass: egui_wgpu::renderer::RenderPass,
    pub widgets: Vec<Box<dyn CtWidget>>,
}

impl ControlPanel {
    pub fn new<T>(ctx: &RenderContext, format: wgpu::TextureFormat, el: &EventLoop<T>) -> Self {
        Self {
            egui_context: Default::default(),
            egui_winit_state: egui_winit::State::new(el),
            egui_pass: EguiRenderPass::new(&ctx.device, format, 1),
            widgets: vec![],
        }
    }

    pub fn frame(&mut self, info: &FrameInfo, engine: &mut Engine) -> Vec<wgpu::CommandBuffer> {
        if info.is_on_first_frame() {
            if !engine.game_root.is_invalid() {
                self.widgets.push(Box::new(SideView));
                self.widgets.push(Box::new(TopView));
            } else {
                self.widgets.push(Box::new(RootSelectWindow::default()));
            }
        }

        let mut new_widgets = vec![];
        let mut command_buffers = vec![];

        let full_output = self.egui_context.run(
            self.egui_winit_state.take_egui_input(&engine.window),
            |ctx| {
                for widget in &mut self.widgets {
                    let mut response = widget.show(ctx, info, engine);
                    new_widgets.append(&mut response.new_widgets);
                    command_buffers.append(&mut response.pre_buffers);
                }
            },
        );

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

        let rcontext = &engine.renderer.context;

        let mut encoder = rcontext
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Egui Command Encoder"),
            });

        let wsize = engine.window.inner_size();
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [wsize.width, wsize.height],
            pixels_per_point: engine.window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_pass
                .update_texture(&rcontext.device, &rcontext.queue, *id, image_delta);
        }

        let tesselated = self.egui_context.tessellate(full_output.shapes.clone());

        self.egui_pass.update_buffers(
            &rcontext.device,
            &rcontext.queue,
            &tesselated,
            &screen_descriptor,
        );

        let view = &engine.renderer.main_window.current.as_ref().expect("no current frame").view;

        self.egui_pass.execute(
            &mut encoder,
            view,
            &tesselated,
            &screen_descriptor,
            Some(wgpu::Color::BLACK),
        );

        for id in &full_output.textures_delta.free {
            self.egui_pass.free_texture(id);
        }

        command_buffers.push(encoder.finish());
        command_buffers
    }
}

#[derive(Default)]
pub struct CtResponse {
    pub new_widgets: Vec<Box<dyn CtWidget>>,
    pub pre_buffers: Vec<wgpu::CommandBuffer>,
}

impl CtResponse {
    pub fn add_info_box(&mut self, title: &str, message: &str) {
        self.new_widgets
            .push(Box::new(MessageBox::info(title, message)));
    }

    pub fn add_warn_box(&mut self, title: &str, message: &str) {
        self.new_widgets
            .push(Box::new(MessageBox::warn(title, message)));
    }

    pub fn add_error_box(&mut self, title: &str, message: &str) {
        self.new_widgets
            .push(Box::new(MessageBox::error(title, message)));
    }

    pub fn add_widget(&mut self, unboxed_widget: impl CtWidget) {
        self.new_widgets.push(Box::new(unboxed_widget));
    }

    pub fn make_widget<T: CtWidget + Default>(&mut self) {
        self.new_widgets.push(Box::new(T::default()));
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

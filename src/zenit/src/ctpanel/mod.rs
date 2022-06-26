//! Control panel is the main engine interface, hosting the actual game within
//! a tiny window. Call it a developer UI if you want to.
use crate::{
    main_window::MainWindow,
    render::{
        self,
        base::{surface::RenderWindow, RenderContext},
        RenderCommands,
    },
    schedule::TopFrameStage,
};
use bevy_ecs::prelude::*;

pub use egui_wgpu::renderer::RenderPass as EguiRenderPass;
pub use egui_winit::State as EguiWinitState;

pub mod ext;
pub mod message_box;
pub mod root_select;
pub mod side;
pub mod top;
//pub mod texture_viewer;

#[derive(Component, Default)]
pub struct Widget;

pub fn init(world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(
        TopFrameStage::FrameStart,
        frame_start_system.after(render::begin_frame_system),
    );
    schedule.add_system_to_stage(
        TopFrameStage::FrameFinish,
        frame_end_system.before(render::finish_frame_system),
    );

    message_box::init(world, schedule);
    root_select::init(world, schedule);
    side::init(world, schedule);
    top::init(world, schedule);

    world.init_resource::<egui::Context>();

    let context = world.get_resource::<RenderContext>().unwrap();
    let window = world.get_resource::<RenderWindow>().unwrap();
    let render_pass = EguiRenderPass::new(&context.device, window.surface_format, 1);

    // TODO: once egui-wgpu allows Send + Sync, use insert_resource instead
    // The egui-wgpu RenderPass implementation includes a non-concurrent
    // TypeMap, which makes the whole otherwise Send + Sync structure screwed
    // in multithreading. Hopefully Bevy's ECS handles this well lol.
    world.insert_non_send_resource(render_pass);

    // TODO: better egui+winit integration stuff
    // Note: at the time of writing, egui_winit also took an &EventLoop param,
    // to attempt getting an optional Wayland handle, only to get clipboard
    // support for this platform. No idea wtf is happening, but this needs to
    // be looked into, since winit isn't properly integrated into egui here.
    // And yes, clipboard is otherwise broken on other platforms (cursors too)
    world.insert_resource(EguiWinitState::new_with_wayland_display(None));
}

pub fn frame_start_system(
    mut winit_state: ResMut<EguiWinitState>,
    egui_context: ResMut<egui::Context>,
    main_window: Res<MainWindow>,
) {
    let inputs = winit_state.take_egui_input(&main_window);
    egui_context.begin_frame(inputs);
}

pub fn frame_end_system(
    mut commands: ResMut<RenderCommands>,
    mut egui_pass: NonSendMut<EguiRenderPass>,
    egui_context: ResMut<egui::Context>,
    render_context: Res<RenderContext>,
    main_window: Res<MainWindow>,
    render_window: Res<RenderWindow>,
) {
    let full_output = egui_context.end_frame();

    let mut encoder =
        render_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Egui Command Encoder"),
            });

    let wsize = main_window.inner_size();
    let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
        size_in_pixels: [wsize.width, wsize.height],
        pixels_per_point: main_window.scale_factor() as f32,
    };

    for (id, image_delta) in &full_output.textures_delta.set {
        egui_pass.update_texture(
            &render_context.device,
            &render_context.queue,
            *id,
            image_delta,
        );
    }

    let tesselated = egui_context.tessellate(full_output.shapes.clone());

    egui_pass.update_buffers(
        &render_context.device,
        &render_context.queue,
        &tesselated,
        &screen_descriptor,
    );

    let view = &render_window
        .current
        .as_ref()
        .expect("no current frame")
        .view;

    egui_pass.execute(
        &mut encoder,
        view,
        &tesselated,
        &screen_descriptor,
        Some(wgpu::Color::BLACK),
    );

    for id in &full_output.textures_delta.free {
        egui_pass.free_texture(id);
    }

    commands.push(encoder.finish());
}

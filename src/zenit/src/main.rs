// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{
    ctpanel::{root_select::RootSelectWindowBundle, EguiWinitState},
    main_window::MainWindow,
    render::base::{surface::RenderWindow, RenderContext},
    root::GameRoot,
};
use bevy_ecs::prelude::*;
use clap::Parser;
use log::*;
use std::sync::Arc;
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod assets;
pub mod cli;
pub mod ctpanel;
pub mod main_window;
pub mod platform;
pub mod profiling;
pub mod render;
pub mod root;
pub mod schedule;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: move this game loop code somewhere lmao

pub fn main() -> ! {
    pretty_env_logger::formatted_builder()
        .format_indent(None)
        .format_timestamp(None)
        .filter_level(LevelFilter::Trace)
        .filter_module("wgpu_hal", LevelFilter::Off)
        .filter_module("wgpu_core", LevelFilter::Error)
        .filter_module("naga", LevelFilter::Off)
        .init();

    let args = cli::Args::parse();

    info!("Welcome to Zenit Engine {}", VERSION);

    #[cfg(feature = "crash-handler")]
    crash::enable_panic_handler();

    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Zenit Engine")
            .with_inner_size(LogicalSize::new(1280i32, 720i32))
            .build(&event_loop)
            .expect("Couldn't create main window"),
    );

    let mut world = World::new();
    let mut schedule = schedule::create_top_scheduler();

    let game_root = GameRoot::new(args.game_root.as_ref());
    if game_root.is_invalid() {
        world
            .spawn()
            .insert_bundle(RootSelectWindowBundle::default());
    }

    world.insert_resource(MainWindow(window.clone()));
    world.insert_resource(game_root);
    world.insert_resource(args);

    profiling::frame_profiler::init(&mut world, &mut schedule);
    render::init(&mut world, &mut schedule);
    ctpanel::init(&mut world, &mut schedule);

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    // It's a bit hacky, but it works
                    let context = world.remove_resource::<RenderContext>().unwrap();
                    world
                        .get_resource_mut::<RenderWindow>()
                        .unwrap()
                        .on_resize(&context.device);
                    world.insert_resource(context);
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");
                *flow = ControlFlow::Exit;

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                window.set_visible(false);
            }
            event => {
                // Hack no. 2
                // This (in)conveniently stays here because of window events' lifetiming
                let mut winit_state = world.remove_resource::<EguiWinitState>().unwrap();
                let egui_context = world.get_resource::<egui::Context>().unwrap();
                winit_state.on_event(&egui_context, &event);
                world.insert_resource(winit_state);
            }
        },
        Event::MainEventsCleared => {
            schedule.run(&mut world);
        }
        _ => {}
    });
}

// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{
    engine::{Engine, FrameInfo},
    profiling::FrameProfiler,
    render::Renderer,
    root::GameRoot, main_window::MainWindow,
};
use bevy_ecs::prelude::*;
use clap::Parser;
use glam::UVec2;
use log::*;
use std::{ops::Deref, sync::Arc};
use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::*,
    window::{Window, WindowBuilder},
};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod assets;
pub mod cli;
pub mod ctpanel;
pub mod engine;
pub mod platform;
pub mod profiling;
pub mod render;
pub mod root;
pub mod schedule;
pub mod main_window;

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

    //let renderer = render::Renderer::new(window.clone()).unwrap();

    let mut world = World::new();
    let mut schedule = schedule::create_top_scheduler();

    world.insert_resource(MainWindow(window.clone()));
    world.insert_resource(args);
    
    render::init(&mut world, &mut schedule);
    ctpanel::init(&mut world, &mut schedule);

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    let mut renderer = world.get_resource_mut::<Renderer>().unwrap();
                    let context = renderer.context.clone();
                    renderer.main_window.reconfigure(
                        &context.device,
                        UVec2::new(new_size.width, new_size.height),
                    );
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
                //panel.egui_winit_state.on_event(&panel.egui_context, &event);
            }
        },
        Event::MainEventsCleared => {
            schedule.run(&mut world);
        }
        _ => {}
    });
}

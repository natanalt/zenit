// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{root::GameRoot, engine::Engine, scene::SceneSystem};
use clap::Parser;
use log::*;
use std::sync::Arc;
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod cli;
pub mod platform;
pub mod root;
pub mod scene;
pub mod engine;

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

    info!("Welcome to Zenit Engine {VERSION}");

    #[cfg(feature = "crash-handler")]
    crash::enable_panic_handler();

    let eloop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Zenit Engine")
            .with_inner_size(LogicalSize::new(1280i32, 720i32))
            .build(&eloop)
            .expect("Couldn't create main window"),
    );

    let game_root = GameRoot::new(args.game_root.as_ref());
    
    let engine = Engine::builder()
        .make_system::<SceneSystem>()
        .with_data(Arc::new(args))
        .build()
        .run();

    // The main thread gets hijacked as the windowing thread
    eloop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    // .. //
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");
                *flow = ControlFlow::Exit;

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                window.set_visible(false);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            // ... //
        }
        _ => {}
    });
}

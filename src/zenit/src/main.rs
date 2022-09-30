// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{engine::Engine, render::RenderSystem, root::GameRoot, scene::SceneSystem};
use clap::Parser;
use log::*;
use std::{
    hint,
    sync::{atomic::Ordering, Arc},
};
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod cli;
pub mod engine;
pub mod platform;
pub mod render;
pub mod root;
pub mod scene;

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

    let game_root = GameRoot::new(args.game_root.as_ref());

    let eloop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Zenit Engine")
            .with_inner_size(LogicalSize::new(1280i32, 720i32))
            .build(&eloop)
            .expect("Couldn't create main window"),
    );

    let comms = Engine::builder()
        .make_system::<SceneSystem>()
        .make_system::<RenderSystem>()
        .with_data(args)
        .with_data(game_root) // TODO: change to mutex or rwlock?
        .with_data(window.clone())
        .build_and_run();

    // The main thread gets hijacked as the windowing thread
    eloop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                window.set_visible(false);

                comms.should_run.store(false, Ordering::SeqCst);
                *flow = ControlFlow::Exit;
            }

            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                let _ = new_inner_size;
                // TODO: handle scale factor changes explicitly
                warn!("Scale factor changed to {scale_factor}, the engine doesn't handle this yet");
            }

            event => {
                let result = comms.event_sender.send(event.to_static().unwrap());

                // Is this really the best way to independently detect engine shutdowns? lol
                if let Err(error) = result {
                    if comms.is_running.load(Ordering::SeqCst) {
                        panic!("couldn't send events to engine: {error:?}")
                    } else {
                        *flow = ControlFlow::Exit;
                    }
                }
            }
        },

        Event::LoopDestroyed => {
            while comms.is_running.load(Ordering::SeqCst) {
                hint::spin_loop();
            }

            trace!("Engine reported shutdown, shutting down the event loop");
        }
        _ => {}
    });
}

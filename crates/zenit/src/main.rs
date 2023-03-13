// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{
    assets::{manager::AssetManager, root::GameRoot},
    render::system::RenderSystem,
    scene::system::SceneSystem,
};
use clap::Parser;
use log::*;
use parking_lot::Mutex;
use std::sync::{atomic::Ordering, Arc};
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod assets;
pub mod cli;
pub mod ecs;
pub mod engine;
pub mod platform;
pub mod render;
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

    if args.singlethreaded {
        // TODO: --singlethreaded support
        warn!("Singlethreaded engine execution is not yet supported");
    }

    let game_root = GameRoot::new(args.game_root.as_ref());

    let eloop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Zenit Engine")
            .with_inner_size(LogicalSize::new(1280i32, 720i32))
            .build(&eloop)
            .expect("couldn't create main window"),
    );

    let captured_window = window.clone();
    let (engine_context, engine_thread_handle) = engine::start(move |builder| {
        let gc = builder.global_context();
        gc.game_root = game_root;
        gc.asset_manager = Some(Arc::new(Mutex::new(AssetManager::new(
            gc.game_root.clone(),
        ))));

        builder
            .with_system(SceneSystem::new())
            .with_system(RenderSystem::new(captured_window));
    });

    let mut engine_thread_handle = Some(engine_thread_handle);

    // The main thread gets hijacked as the windowing thread
    eloop.run(move |event, _, flow| match event {
        Event::NewEvents(_) => {
            if !engine_context.is_running.load(Ordering::Acquire) {
                flow.set_exit();
            }
        }

        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                // it just feels better
                window.set_visible(false);

                engine_context.request_shutdown();
                engine_thread_handle.take().unwrap().join().unwrap();

                flow.set_exit();
            }

            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                let _ = new_inner_size;
                // TODO: handle scale factor changes explicitly
                warn!("Scale factor changed to {scale_factor}, the engine doesn't handle this yet");
            }

            event => {}
        },

        Event::MainEventsCleared => {}

        Event::LoopDestroyed => {
            trace!("Shutting down the event loop");
        }
        _ => {}
    });
}

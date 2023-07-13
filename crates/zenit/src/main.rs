// On Windows, the default subsystem is "console", which includes a ✨console✨
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{
    assets::{AssetLoader, AssetManager, GameRoot},
    graphics::{system::RenderSystem, Renderer},
    scene::{system::SceneSystem, EngineBorrow}, entities::Universe,
};
use clap::Parser;
use log::*;
use std::sync::{atomic::Ordering, Arc};
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod assets;
pub mod cli;
pub mod devui;
pub mod engine;
pub mod entities;
pub mod graphics;
pub mod platform;
pub mod scene;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    if args.single_thread {
        // TODO: --single-thread support
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
        
        let window = captured_window.clone();
        
        let globals = builder.global_state();
        let (mut renderer, render_system_dc, surface, sconfig) = Renderer::new(&window);
        let mut assets = AssetManager::new(game_root.clone(), &mut renderer);
        let mut universe = Universe::new();
        
        let mut engine = EngineBorrow {
            globals,
            assets: &mut assets,
            renderer: &mut renderer,
            universe: &mut universe,
        };

        AssetLoader::new(&mut engine)
            .load_builtins()
            .expect("could not load built-in assets");

        builder
            .with_system(RenderSystem::new(
                &mut renderer,
                render_system_dc,
                window.clone(),
                surface,
                sconfig,
            ))
            .with_system(SceneSystem::new());
    
        let gc = builder.global_state();
        gc.add_any(captured_window);
        gc.add_any(game_root);
        gc.add_lockable(assets);
        gc.add_lockable(renderer);
        gc.add_rw_lockable(universe);
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

            WindowEvent::ScaleFactorChanged { .. } => {}

            event => {
                engine_context
                    .window_events
                    .lock()
                    .push(event.to_static().unwrap());
            }
        },

        Event::MainEventsCleared => {}

        Event::LoopDestroyed => {
            trace!("Shutting down the event loop");
        }
        _ => {}
    });
}

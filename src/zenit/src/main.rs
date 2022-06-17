// On Windows, the default subsystem is "console", which includes a ✨console✨
// (at least by default)
//
// Specifying the subsystem as "windows" disables this
#![cfg_attr(feature = "no-console", windows_subsystem = "windows")]

use crate::{
    engine::{Engine, FrameInfo},
    profiling::FrameProfiler,
    root::GameRoot,
};
use clap::Parser;
use glam::UVec2;
use log::*;
use std::sync::Arc;
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

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

    let frame_profiler = FrameProfiler::new();
    let renderer = render::Renderer::new(window.clone()).unwrap();

    let mut panel = ctpanel::ControlPanel::new(
        &renderer.context,
        renderer.main_window.surface_format,
        &event_loop,
    );

    let mut engine = Engine {
        renderer,
        window,
        frame_profiler,
        game_root: GameRoot::new(args.game_root.as_ref()),
        args,
    };

    let mut frame_info = FrameInfo {
        delta: std::time::Duration::from_millis(1000 / 60),
        frame_count: 0,
    };

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == engine.window.id() => match event {
            WindowEvent::Resized(new_size) => {
                if new_size.width != 0 && new_size.height != 0 {
                    engine.renderer.main_window.reconfigure(
                        &engine.renderer.context.device,
                        UVec2::new(new_size.width as _, new_size.height as _),
                    );
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");
                *flow = ControlFlow::Exit;

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                engine.window.set_visible(false);
            }
            event => {
                panel.egui_winit_state.on_event(&panel.egui_context, &event);
            }
        },
        Event::MainEventsCleared => {
            let frame_time = profiling::measure_time(|| {
                let profiler_frame = profiling::FrameEntry::default();

                engine.renderer.begin_frame();
                let commands = panel.frame(&frame_info, &mut engine);
                engine.renderer.finish_frame(commands).unwrap();

                engine.frame_profiler.push_frame(profiler_frame);

                *flow = ControlFlow::Poll;
                frame_info.frame_count += 1;
            });

            frame_info.delta = frame_time;
        }
        _ => {}
    });
}

use crate::engine::{Engine, FrameInfo};
use log::*;
use std::{sync::Arc, time::Instant};
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

#[cfg(feature = "crash-handler")]
pub mod crash;

pub mod ctpanel;
pub mod engine;
pub mod render;

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

    let mut renderer = render::Renderer::new(window.clone()).unwrap();

    renderer.screens.push(render::base::screen::Screen {
        label: Some("dab".into()),
        target: Arc::new(render::base::texture::Texture2D::new(
            &renderer.context.device,
            glam::IVec2::new(800, 600),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::RENDER_ATTACHMENT |
            wgpu::TextureUsages::TEXTURE_BINDING,
        )),
        layers: vec![Arc::new(
            render::layers::example::TriangleLayer::new(
                &renderer.context,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            )
            .unwrap(),
        )],
    });

    let mut panel = ctpanel::ControlPanel::new(&renderer.context.device, &event_loop);
    let mut engine = Engine { renderer, window };

    let mut frame_info = FrameInfo {
        delta: std::time::Duration::from_millis(1000 / 60),
        frame_count: 0,
    };

    event_loop.run(move |event, _, flow| match event {
        Event::WindowEvent { window_id, event } if window_id == engine.window.id() => match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                *flow = ControlFlow::Exit;

                // Hide the window here so it disappears faster, while the app
                // actually shuts down in the background
                engine.window.set_visible(false);
            }
            event => {
                panel
                    .egui_manager
                    .winit_state
                    .write()
                    .unwrap()
                    .on_event(&panel.egui_manager.context, &event);
            }
        },
        Event::MainEventsCleared => {
            let frame_start = Instant::now();
            
            panel.frame(&frame_info, &mut engine);
            engine.renderer.render_frame().unwrap();
            *flow = ControlFlow::Poll;
            frame_info.frame_count += 1;

            let frame_end = Instant::now();
            let frame_time = frame_end.duration_since(frame_start);
            frame_info.delta = frame_time;
        }
        _ => {}
    });
}

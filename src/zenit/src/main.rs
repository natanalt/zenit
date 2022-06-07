use log::*;
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

pub mod crash;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

    crash::set_panic_hook();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Zenit Engine")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .expect("Couldn't create main window")
        .into();

    let mut renderer = zenit_render::Renderer::new(window).unwrap();
    renderer.screens.push(zenit_render::base::screen::Screen {
        label: Some("dab".into()),
        target: renderer.main_window.clone(),
        layers: vec![Box::new(
            zenit_render::example::TriangleLayer::new(
                &renderer.context,
                renderer.main_window.surface_format,
            )
            .unwrap(),
        )],
    });

    //let main_window = renderer.context.main_window.clone();
    //renderer.add_screen(zenit_render::rcore::screen::Screen::new(&renderer.context, main_window).into());
    info!("Renderer is up and running");

    event_loop.run(move |event, _, flow| match event {
        Event::NewEvents(_) => {}
        Event::WindowEvent { window_id: _, event } =>
            //if window_id == renderer.main_window_surface.window().id() =>
        {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");
                    *flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
        Event::MainEventsCleared => {
            renderer.render_frame().unwrap();
            *flow = ControlFlow::Poll;
        }
        _ => {}
    });
}

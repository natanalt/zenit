use bevy_ecs::prelude::*;
use log::*;
use winit::{dpi::LogicalSize, event::*, event_loop::*, window::WindowBuilder};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(StageLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineStage {
    Update,
}

pub fn main() -> ! {
    pretty_env_logger::formatted_builder()
        .format_indent(None)
        .format_timestamp(None)
        .filter_level(LevelFilter::Trace)
        .init();

    info!("Welcome to Zenit Engine {}", VERSION);

    let mut world = World::default();
    let mut schedule = Schedule::default();
    schedule.add_stage(EngineStage::Update, SystemStage::parallel().with_system(|| {
        info!("o w o");
    }));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Zenit Engine")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .expect("couldn't create main window");

    event_loop.run(move |event, _, flow| match event {
        Event::NewEvents(_) => {}
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                *flow = ControlFlow::Exit;
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            schedule.run(&mut world);
        }
        _ => {}
    });
}

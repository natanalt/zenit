use crate::renderer::Renderer;
use log::info;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy)]
pub enum LoadingTarget {
    /// Aka. the menu
    Shell,
    Ingame,
}

#[derive(Debug, Clone)]
pub enum GameState {
    /// Aka. the menu
    Shell,
    Ingame,
    Loading(LoadingTarget),
}

pub struct Engine {
    exit_requested: bool,
    min_frame_time: Duration,

    current_state: Option<GameState>,

    renderer: Option<Arc<Renderer>>,
}

impl Engine {
    /// Creates a fresh Engine instance
    pub fn new() -> Self {
        Self {
            exit_requested: false,
            current_state: None,
            min_frame_time: Duration::from_secs_f32(1.0 / 60.0),
            renderer: None,
        }
    }

    /// Requests the engine to exit on the next frame
    pub fn exit(&mut self) {
        info!("Exit requested");
        self.exit_requested = true;
    }
}

pub fn run() {
    info!("Welcome to Zenit {}!", VERSION);

    let mut engine = Engine::new();

    let mut frame_start = Instant::now();
    let mut _delta = Duration::ZERO;
    let mut total_runtime = Duration::ZERO;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(&format!("Zenit Engine {}", VERSION))
        .build(&event_loop)
        .expect("couldn't create the window");

    event_loop.run(move |event, _, cf| {
        match event {
            Event::NewEvents(cause) => {
                frame_start = Instant::now();

                if cause == StartCause::Init {
                    info!("Initializing the game...");
                    engine.renderer = Some(Arc::new(pollster::block_on(Renderer::new(&window))));
                }
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        engine.exit();
                    }
                    //WindowEvent::Resized(_physical_size) => {
                    //resize(*physical_size);
                    //}
                    //WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    //resize(**new_inner_size);
                    //}
                    _ => {}
                }
            }

            Event::MainEventsCleared => {
                // Updating, rendering, etc. goes here

                _delta = Instant::now().duration_since(frame_start);
                total_runtime += _delta;
                if _delta < engine.min_frame_time {
                    let sleep_time = engine.min_frame_time - _delta;
                    *cf = ControlFlow::WaitUntil(Instant::now() + sleep_time);
                }
            }

            _ => {}
        }

        if engine.exit_requested {
            *cf = ControlFlow::Exit;
        }
    });
}

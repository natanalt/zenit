//! Root of the engine structure and the event loop
//!

use crate::{
    args::ZenitArgs, console::Console, devui::DevUI, loading::LoadingState, renderer::Renderer,
    resources::Resources, shell::ShellState,
};
use clap::StructOpt;
use glam::IVec2;
use log::info;
use pollster::FutureExt;
use std::{
    iter, mem,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub enum GameState {
    Invalid,
    Shell(ShellState),
    Ingame,
    Loading(LoadingState),
}

#[derive(Debug, Default)]
pub struct Events {
    pub new_size: Option<IVec2>,
}

impl Events {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn resized(&self) -> bool {
        self.new_size.is_some()
    }
}

pub struct Engine {
    exit_requested: bool,
    current_state: GameState,

    pub cli_args: ZenitArgs,

    pub events: Events,
    pub console: Console,
    pub resources: Resources,
    pub devui: Option<Box<DevUI>>,
    pub renderer: Option<Box<Renderer>>,

    pub frame_start: Instant,
    pub min_frame_time: Duration,
    pub delta: Duration,
    pub total_runtime: Duration,
}

impl Engine {
    /// Creates a fresh Engine instance
    pub fn new(cli_args: ZenitArgs, renderer: Renderer, window: &Window) -> Self {
        let resources = Resources::new(&cli_args.game_root);
        Self {
            exit_requested: false,
            current_state: GameState::Loading(LoadingState::shell_loader(&resources)),

            events: Events::default(),
            console: Console::new(),
            resources,
            devui: Some(Box::new(DevUI::new(&renderer, window))),
            renderer: Some(Box::new(renderer)),

            cli_args: ZenitArgs::parse(),

            frame_start: Instant::now(), // an elaborate lie
            min_frame_time: Duration::from_secs_f32(1.0 / 60.0),
            delta: Duration::ZERO,
            total_runtime: Duration::ZERO,
        }
    }

    /// Requests the engine to exit on the next frame
    pub fn exit(&mut self) {
        info!("Exit requested");
        self.exit_requested = true;
    }
}

/// Creates an Engine instance and begins the main loop.
///
/// The main loop is managed entirely by winit; this function will not return.
pub fn run() -> ! {
    let cli_args = ZenitArgs::parse();

    info!("Welcome to Zenit {}!", VERSION);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(&format!("Zenit Engine {}", VERSION))
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .expect("couldn't create the window");

    info!("Game root: {}", cli_args.game_root.display());

    let mut engine = Engine::new(cli_args, Renderer::new(&window).block_on(), &window);

    event_loop.run(move |event, _, cf| {
        let self_window_id = window.id();
        match event {
            Event::NewEvents(_) => {
                engine.frame_start = Instant::now();
                engine.events.reset();
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self_window_id => {
                engine.devui.as_mut().unwrap().process_event(event);
                match event {
                    WindowEvent::CloseRequested => {
                        engine.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        engine.events.new_size = Some(IVec2::new(
                            physical_size.width as _,
                            physical_size.height as _,
                        ));
                    }
                    _ => {}
                }
            }

            Event::MainEventsCleared => {
                // Very hacky workaround to let the renderer take mutable Engine references
                let mut renderer = engine
                    .renderer
                    .take()
                    .expect("Renderer is not present for some reason");
                renderer.begin_frame(&mut engine);
                engine.renderer = Some(renderer);

                // Update the actual game
                engine.current_state =
                    match mem::replace(&mut engine.current_state, GameState::Invalid) {
                        GameState::Shell(state) => state.frame(&mut engine),
                        GameState::Ingame => todo!(),
                        GameState::Loading(state) => state.frame(&mut engine),
                        GameState::Invalid => panic!("Invalid game state"),
                    };

                // egui rendering, also using that terrible workaround
                let mut devui = engine.devui.take().expect("DevUI is missing");
                if let Some(buffer) = devui.frame(&mut engine, &window) {
                    engine.renderer.as_mut().unwrap().buffers.push(buffer);
                }
                engine.devui = Some(devui);

                // Very hacky workaround part 2
                let mut renderer = engine
                    .renderer
                    .take()
                    .expect("Renderer is not present for some reason");
                renderer.finish_frame(&mut engine);
                engine.renderer = Some(renderer);

                // Calculate delta and increment total_runtime
                engine.delta = Instant::now().duration_since(engine.frame_start);
                engine.total_runtime += engine.delta;
                if engine.delta < engine.min_frame_time {
                    let sleep_time = engine.min_frame_time - engine.delta;
                    *cf = ControlFlow::WaitUntil(Instant::now() + sleep_time);
                }
            }

            _ => {}
        }

        if engine.exit_requested {
            *cf = ControlFlow::Exit;
        }
    })
}

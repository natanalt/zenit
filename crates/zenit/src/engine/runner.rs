use super::{EngineContext, GlobalState, System};
use crate::engine::{EngineBus, FrameProfiler};
use log::*;
use once_cell::sync::OnceCell;
use std::{
    sync::{atomic::Ordering, Arc, Barrier},
    thread::{self, JoinHandle},
};

/// Starts the core engine in its own dedicated thread.
///
/// The provided parameter is a closure, which will receive an [`EngineBuilder`], capable of spawning
/// systems and their threads. This closure is executed inside the engine thread.
///
/// This function will block until the engine thread finishes early initialization and provides
/// an [`Arc`] of [`EngineContext`] that is eventually returned by this function. This context struct
/// is used by the windowing event loop to communicate with the engine.
pub fn start(
    f: impl FnOnce(&mut EngineBuilder) + Send + Sync + 'static,
) -> (Arc<EngineContext>, JoinHandle<()>) {
    // Because the engine thread has to initialize the context, the sync::OnceCell is used
    // as the synchronization primitive. After spawning the thread, this function will
    // await the setup of this cell.
    let ec_cell1: Arc<OnceCell<Arc<EngineContext>>> = Arc::new(OnceCell::new());
    let ec_cell2 = ec_cell1.clone();

    let join_handle = thread::Builder::new()
        .name(String::from("Engine Controller"))
        .spawn(move || {
            // Similarly to the ec_cell, this cell is observed by all system worker threads.
            // Once all system threads are in place, this value is constructed and provided to
            // all system threads.
            let input_cell: OnceCell<SystemThreadInputs> = OnceCell::new();

            // Because of lifetime requirements, these values are stored here.
            // They are properly initialized in the scope itself, so over here meaningless valid
            // defaults are provided.
            let mut engine_context = Default::default();
            let mut stage_barrier = Barrier::new(0);

            thread::scope(|scope| {
                // Early initialization
                // ----------------------------------------------------------------
                let mut raw_ec = EngineContext::default();
                let mut frame_profiler = FrameProfiler::new();
                let mut total_threads = 0;

                raw_ec.globals.get_mut().add_any(EngineBus::new());
                raw_ec
                    .globals
                    .get_mut()
                    .add_any(frame_profiler.history.clone());

                f(&mut EngineBuilder {
                    scope: &scope,

                    frame_profiler: &mut frame_profiler,
                    total_threads: &mut total_threads,
                    engine_context: &mut raw_ec,

                    input_cell: &input_cell,
                });

                engine_context = Arc::new(raw_ec);
                stage_barrier = Barrier::new(total_threads + 1);

                // Send engine readiness notifications and associated data
                // ----------------------------------------------------------------
                // (note, using get_or_init, as it allows insertions that don't return
                // Result<(), T>, which requires T: Debug, like is the case with set)
                ec_cell2.get_or_init(|| engine_context.clone());
                input_cell.get_or_init(|| SystemThreadInputs {
                    stage_barrier: &stage_barrier,
                    engine_context: &engine_context,
                });

                // Begin the engine loop proper
                // ----------------------------------------------------------------
                trace!("Starting the engine controller loop");
                let ec = &engine_context;
                loop {
                    frame_profiler.begin_frame();

                    stage_barrier.wait(); // Frame initialization
                    stage_barrier.wait(); // Main processing
                    stage_barrier.wait(); // Post processing

                    // Lock the bus, update window events, forward it to the next frame
                    {
                        let mut events = ec.window_events.lock();
                        let mut gs = ec.globals.write();
                        let bus = gs.get_mut::<EngineBus>();
                        bus.send_messages(events.drain(..).map(|event| Box::from(event) as _));
                        bus.next_frame();
                    }

                    frame_profiler.finish_frame();

                    if !ec.should_run.load(Ordering::Acquire) {
                        trace!("Shutting down the engine controller loop");
                        ec.is_running.store(false, Ordering::Release);
                        stage_barrier.wait(); // End of frame
                        break;
                    }

                    stage_barrier.wait(); // End of frame
                }
                trace!("Engine controller loop finished");
            })
        })
        .unwrap();

    (ec_cell1.wait().clone(), join_handle)
}

/// Input data sent to all system threads via a dedicated cell.
struct SystemThreadInputs<'a> {
    stage_barrier: &'a Barrier, // Move this to EngineContext maybe?
    engine_context: &'a EngineContext,
}

/// Engine builder, given to the closure provided in [`start`]. Provides functionality for setting
/// up initial engine state.
///
/// ## Internal lifetimes
/// what a clusterfuck
///  * `'init` - encompasses the scope of [`EngineContext`] creation
///  * `'scope` - encompasses the [`thread::scope`] scope
///  * `'env` - encompasses the engine thread data from outside the [`thread::scope`]
///
/// System threads can only reference `'env` data.
pub struct EngineBuilder<'init, 'scope, 'env> {
    scope: &'scope thread::Scope<'scope, 'env>,

    frame_profiler: &'init mut FrameProfiler,
    total_threads: &'init mut usize,
    engine_context: &'init mut EngineContext,

    input_cell: &'env OnceCell<SystemThreadInputs<'env>>,
}

impl<'init, 'scope, 'env> EngineBuilder<'_, '_, '_> {
    pub fn global_state(&mut self) -> &mut GlobalState {
        self.engine_context.globals.get_mut()
    }

    pub fn with_system<S: System>(&mut self, mut system: S) -> &mut Self {
        let input_cell = self.input_cell;

        let system_profiler = self.frame_profiler.add_system(system.label());
        system.init(self.engine_context);

        trace!("Starting worker thread for system `{}`", system.label());
        self.scope.spawn(move || {
            let mut first_frame_called = false;

            let SystemThreadInputs {
                stage_barrier,
                engine_context,
            } = input_cell.wait().clone();

            loop {
                let mut sp = system_profiler.lock();

                sp.frame_init(|| {
                    if !first_frame_called {
                        first_frame_called = true;
                        system.first_frame(engine_context);
                    }
                    system.frame_initialization(engine_context)
                });
                stage_barrier.wait(); // Frame initialization

                let gc = engine_context.globals.read();
                sp.main_process(|| system.main_process(engine_context, &gc));
                drop(gc);
                stage_barrier.wait(); // Main processing

                sp.post_process(|| system.post_process(engine_context));
                stage_barrier.wait(); // Post processing

                drop(sp);

                stage_barrier.wait(); // End of frame

                if !engine_context.is_running.load(Ordering::Acquire) {
                    break;
                }
            }
        });

        *self.total_threads += 1;
        self
    }
}

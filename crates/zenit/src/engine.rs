//! The core engine scheduling
//!
//! Zenit Engine tries to do basic multithreading with the top level engine modules. This allows
//! the main processing code, rendering code, event loop, etc. to be executed in parallel.
//!
//! Each one of those top level modules implements a trait called [`System`], and is futher referred
//! to as a system. They are synchronized and managed by the Engine Controller thread. During each
//! frame, processing of all systems is split into the following stages:
//!  * Initialization
//!  * Main Processing
//!  * Post Processing
//!
//! Each of these stages is a barrier, so for example, post processing only begins once all systems
//! have finished their main processing.
//!
//! ## Notes for implementing systems
//!  * Try to not assume that each system executes in parallel. While this is the case now, in the
//!    future a singlethreaded mode may be implemented, in which systems are executed sequentially
//!    for each phase.
//!

use crate::{ecs::Universe, render::api::Renderer, assets::{root::GameRoot, manager::AssetManager}};
use log::*;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use smallvec::SmallVec;
use std::{
    any::Any,
    collections::VecDeque,
    fmt::Debug,
    mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// The global context contains data that may require mutability, or isn't related to the enigne's
/// core execution.
///
/// Available in the [`EngineContext`] as a read-write locked resource. During main processing,
/// it is provided as an immutable reference, but systems are allowed to briefly write lock it in
/// other stages of execution.
pub struct GlobalContext {
    pub game_root: GameRoot,
    pub renderer: Option<Arc<Mutex<Renderer>>>,
    pub asset_manager: Option<Arc<Mutex<AssetManager>>>,
    pub universe: Option<Arc<RwLock<Universe>>>,
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self {
            game_root: GameRoot::default(),
            renderer: None,
            asset_manager: None,
            universe: None,
        }
    }
}

impl GlobalContext {
    /// Shorthand for unwrapping and locking the [`Renderer`] mutex.
    pub fn lock_renderer(&self) -> MutexGuard<Renderer> {
        self.renderer
            .as_ref()
            .expect("renderer not initialized")
            .lock()
    }

    /// Shorthand for unwrapping and locking the [`AssetManager`] mutex.
    pub fn lock_asset_manager(&self) -> MutexGuard<AssetManager> {
        self.asset_manager
            .as_ref()
            .expect("asset manager not initialized")
            .lock()
    }

    /// Shorthand for unwrapping and read-locking the [`Universe`] lock.
    pub fn read_universe(&self) -> RwLockReadGuard<Universe> {
        self.universe.as_ref().expect("ECS not initialized").read()
    }

    /// Shorthand for unwrapping and write-locking the [`Universe`] lock.
    pub fn write_universe(&self) -> RwLockWriteGuard<Universe> {
        self.universe.as_ref().expect("ECS not initialized").write()
    }
}

/// Global immutable engine data.
pub struct EngineContext {
    /// See: [`Self::request_shutdown`]
    should_run: AtomicBool,
    /// Flag which determines whether the engine controller is still running.
    ///
    /// This should only be modified by the controller thread. Other code can freely load this value,
    /// though.
    pub is_running: AtomicBool,
    /// See [`GlobalContext`] docs.
    pub global_context: RwLock<GlobalContext>,
}

impl Default for EngineContext {
    fn default() -> Self {
        Self {
            should_run: AtomicBool::new(true),
            is_running: AtomicBool::new(true),
            //global_data: AHashMap::default(),
            global_context: RwLock::new(GlobalContext::default()),
        }
    }
}

impl EngineContext {
    /// Tells the engine controller to break out of its loop at the end of the current frame.
    pub fn request_shutdown(&self) {
        trace!(
            "Thread `{}` requesting shutdown",
            thread::current().name().unwrap_or("(no name)")
        );
        self.should_run.store(false, Ordering::Release);
    }
}

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

                    frame_profiler.finish_frame();

                    if !ec.should_run.load(Ordering::Acquire) {
                        trace!("Shutting down the engine controller loop");
                        ec.is_running.store(false, Ordering::Release);
                        stage_barrier.wait(); // End of frame
                        break;
                    } else {
                        stage_barrier.wait(); // End of frame
                    }
                }
                trace!("Engine controller loop finished");
            })
        })
        .unwrap();

    (ec_cell1.wait().clone(), join_handle)
}

#[derive(Clone)]
struct SystemThreadInputs<'a> {
    stage_barrier: &'a Barrier,
    engine_context: &'a EngineContext,
}

/// Engine builder, used by the closure provided to [`start`].
///
/// ## Lifetimes
/// what a clusterfuck
///  * `'init` - encompasses the scope of [`EngineContext`] creation
///  * `'scope` - encompasses the [`thread::scope`] scope
///  * `'env` - encompasses the engine thread data from outside the [`thread::scope`]
///
/// System threads can only access `'env` data, which over here is just the `input_cell`.
pub struct EngineBuilder<'init, 'scope, 'env> {
    scope: &'scope thread::Scope<'scope, 'env>,

    frame_profiler: &'init mut FrameProfiler,
    total_threads: &'init mut usize,
    engine_context: &'init mut EngineContext,

    input_cell: &'env OnceCell<SystemThreadInputs<'env>>,
}

impl<'init, 'scope, 'env> EngineBuilder<'_, '_, '_> {
    pub fn global_context(&mut self) -> &mut GlobalContext {
        self.engine_context.global_context.get_mut()
    }

    pub fn with_system<S: System>(&mut self, mut system: S) -> &mut Self {
        let input_cell = self.input_cell;

        let system_profiler = self.frame_profiler.add_system(system.label());
        system.init(self.engine_context);

        trace!("Starting worker thread for system `{}`", system.label());
        self.scope.spawn(move || {
            let SystemThreadInputs {
                stage_barrier,
                engine_context,
            } = input_cell.wait().clone();

            loop {
                let mut sp = system_profiler.lock();

                sp.frame_init(|| system.frame_initialization(engine_context));
                stage_barrier.wait(); // Frame initialization

                let gc = engine_context.global_context.read();
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

/// Trait implemented by engine systems, see module docs for details.
pub trait System: Any + Send + Sync {
    /// The system's label, primarily used to identify individual worker threads.
    /// It must be a constant.
    fn label(&self) -> &'static str;

    /// An initialization function called exactly once for each system.
    ///
    /// It is called **inside the engine thread**, sequentially for each system, in order of
    /// registration. This is the only time a system may also take a mutable reference to
    /// the [`EngineContext`].
    fn init(&mut self, ec: &mut EngineContext) {
        let _ = ec;
    }

    /// System frame stage 1.
    fn frame_initialization(&mut self, ec: &EngineContext) {
        let _ = ec;
    }

    /// System frame stage 2.
    fn main_process(&mut self, ec: &EngineContext, gc: &GlobalContext) {
        let _ = (ec, gc);
    }

    /// System frame stage 3.
    fn post_process(&mut self, ec: &EngineContext) {
        let _ = ec;
    }
}

struct FrameProfiler {
    pub max_history_size: usize,
    pub history: VecDeque<FrameTiming>,
    profilers: Vec<Arc<Mutex<SystemProfiler>>>,
    pending: FrameTiming,
}

impl FrameProfiler {
    fn new() -> Self {
        Self {
            // Should amount to ~80 seconds @ 60 FPS, ~34 seconds @ 144 FPS
            // This could be made reconfigurable via cmdline args, or config or something.
            // This should amount to up to ~2,17 MiB of frame profiler logs
            max_history_size: 5000,
            history: VecDeque::new(),
            profilers: vec![],
            pending: FrameTiming::default(),
        }
    }

    fn add_system(&mut self, label: &'static str) -> Arc<Mutex<SystemProfiler>> {
        let profiler = Arc::new(Mutex::new(SystemProfiler {
            label,
            frame_init_start: None,
            frame_init_end: None,
            main_process_start: None,
            main_process_end: None,
            post_process_start: None,
            post_process_end: None,
        }));

        self.profilers.push(profiler.clone());
        profiler
    }

    fn begin_frame(&mut self) {
        debug_assert!(self.pending.controller_start.is_none());
        self.pending.controller_start = Some(Instant::now());
    }

    fn finish_frame(&mut self) -> &FrameTiming {
        self.pending.controller_end = Some(Instant::now());

        let mut pending = mem::take(&mut self.pending);
        for profiler in &self.profilers {
            let mut sp = profiler.lock();
            pending.system_timings.push(sp.reset());
        }

        if self.history.len() >= self.max_history_size {
            self.history.pop_front();
        }

        self.history.push_back(pending);
        self.history.back().unwrap()
    }
}

#[derive(Default)]
pub struct FrameTiming {
    /// Moment when the engine controller itself began frame processing
    pub controller_start: Option<Instant>,
    /// Moment when the engine controller itself finished frame processing.
    pub controller_end: Option<Instant>,
    /// Individual systems' timing information.
    pub system_timings: SmallVec<[SystemTiming; 5]>,
}

impl Debug for FrameTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameTiming")
            .field(
                "controller_time",
                &calculate_time(self.controller_start, self.controller_end),
            )
            .field("controller_start", &self.controller_start)
            .field("controller_end", &self.controller_end)
            .field("system_timings", &self.system_timings)
            .finish()
    }
}

pub struct SystemTiming {
    pub label: &'static str,
    pub frame_init_start: Instant,
    pub frame_init_end: Instant,

    pub main_process_start: Instant,
    pub main_process_end: Instant,

    pub post_process_start: Instant,
    pub post_process_end: Instant,
}

impl Debug for SystemTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemTiming")
            .field("label", &self.label)
            .field(
                "frame_init_time",
                &(self.frame_init_end - self.frame_init_start),
            )
            .field(
                "main_process_time",
                &(self.main_process_end - self.main_process_start),
            )
            .field(
                "post_process_time",
                &(self.post_process_end - self.post_process_start),
            )
            .field("frame_init_start", &self.frame_init_start)
            .field("frame_init_end", &self.frame_init_end)
            .field("frame_main_process_start", &self.main_process_start)
            .field("frame_main_process_end", &self.main_process_end)
            .field("frame_post_process_start", &self.post_process_start)
            .field("frame_post_process_end", &self.post_process_end)
            .finish()
    }
}

struct SystemProfiler {
    label: &'static str,
    frame_init_start: Option<Instant>,
    frame_init_end: Option<Instant>,
    main_process_start: Option<Instant>,
    main_process_end: Option<Instant>,
    post_process_start: Option<Instant>,
    post_process_end: Option<Instant>,
}

impl SystemProfiler {
    fn frame_init(&mut self, f: impl FnOnce()) {
        debug_assert!(self.frame_init_start.is_none() && self.frame_init_end.is_none());
        self.frame_init_start = Some(Instant::now());
        f();
        self.frame_init_end = Some(Instant::now());
    }

    fn main_process(&mut self, f: impl FnOnce()) {
        debug_assert!(self.main_process_start.is_none() && self.main_process_end.is_none());
        self.main_process_start = Some(Instant::now());
        f();
        self.main_process_end = Some(Instant::now());
    }

    fn post_process(&mut self, f: impl FnOnce()) {
        debug_assert!(self.post_process_start.is_none() && self.post_process_end.is_none());
        self.post_process_start = Some(Instant::now());
        f();
        self.post_process_end = Some(Instant::now());
    }

    fn reset(&mut self) -> SystemTiming {
        SystemTiming {
            label: self.label,
            frame_init_start: self.frame_init_start.take().unwrap(),
            frame_init_end: self.frame_init_end.take().unwrap(),
            main_process_start: self.main_process_start.take().unwrap(),
            main_process_end: self.main_process_end.take().unwrap(),
            post_process_start: self.post_process_start.take().unwrap(),
            post_process_end: self.post_process_end.take().unwrap(),
        }
    }
}

fn calculate_time(start: Option<Instant>, end: Option<Instant>) -> Option<Duration> {
    match (start, end) {
        (Some(start), Some(end)) => Some(end.duration_since(start)),
        _ => None,
    }
}

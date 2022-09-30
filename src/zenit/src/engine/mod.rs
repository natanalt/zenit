//! Engine core stuff

use self::{builder::EngineBuilder, system::System};
use crate::engine::system::SystemContext;
use log::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::{
    any::{Any, TypeId},
    iter,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc, Barrier, Mutex, RwLock,
    },
    thread,
    time::{Duration, Instant},
};
use winit::event::WindowEvent;
use zenit_utils::{MutexExt, RwLockExt};

pub mod builder;
pub mod system;

pub type TypeErasedSystemInterface = Box<dyn Any + Send + Sync>;
pub type SystemInterfaceTypeMap = FxHashMap<TypeId, TypeErasedSystemInterface>;
pub type TypeErasedData = Box<dyn Any + Send + Sync>;
pub type DataTypeMap = FxHashMap<TypeId, TypeErasedData>;

/// Main engine controller.
pub struct Engine {
    /// All systems to be ran by the engine
    systems: Vec<Box<dyn for<'a> System<'a>>>,
    /// Type map holding instances of [`HasSystemInterface::SystemInterface`]
    system_interfaces: SystemInterfaceTypeMap,
    /// Type map holding instances of [`Data`]
    data: DataTypeMap,
    /// Event loop events go here
    event_receiver: Receiver<WindowEvent<'static>>,
    /// Flag signifying whether the game loop should continue. Can be set to
    /// false by the event loop or systems.
    should_run: Arc<AtomicBool>,
    /// Flag signifying whether the engine is still running. Accessible by
    /// the event loop, to determine whether it's okay to shut down or not.
    ///
    /// The event loop shuts down once this flag is false.
    is_running: Arc<AtomicBool>,
}

impl Engine {
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    /// Runs the engine in the current thread, until it exits.
    pub fn run(self) {
        let Self {
            systems,
            system_interfaces,
            data,
            event_receiver,
            should_run,
            is_running,
        } = self;

        let data_lock = RwLock::new(data);
        let data_to_remove = RwLock::new(vec![]);
        let data_to_add = RwLock::new(vec![]);
        // 32 is an arbitrary number, I doubt it will ever be reached tbh
        let events_lock = RwLock::new(Vec::with_capacity(32));

        let stage_controller = TopLevelController::new(systems.len());

        thread::scope(|scope| {
            for mut system in systems {
                trace!("Creating `{}` system instance", system.name());

                // This gets moved into the thread
                let mut stage_sync = stage_controller.next_system(system.name().to_string());

                // Passing references into a `move` closure requires binding
                // them into variables. A bit annoying.
                let thread_references = (
                    &should_run,
                    &system_interfaces,
                    &data_lock,
                    &events_lock,
                    &data_to_add,
                    &data_to_remove,
                );

                thread::Builder::new()
                    .name(system.name().to_string())
                    .spawn_scoped(scope, move || {
                        let (
                            should_run,
                            system_interfaces,
                            data_lock,
                            events_lock,
                            data_to_add,
                            data_to_remove,
                        ) = thread_references;

                        let mut ran_init = false;

                        // System control loop
                        while should_run.load(Ordering::SeqCst) {
                            stage_sync.begin_frame();

                            let events = events_lock.read().unwrap();
                            let data = data_lock.read().unwrap();
                            let mut system_context = SystemContext {
                                should_run: &should_run,
                                system_sync: &mut stage_sync,
                                system_interfaces,
                                data: &data,
                                data_to_remove: &data_to_remove,
                                data_to_add: &data_to_add,
                                events: &events,
                            };

                            if !ran_init {
                                system.init(&mut system_context);
                                ran_init = true;
                            }
                            system.frame(&mut system_context);

                            // Finish frame stage in case the system didn't do it manually
                            stage_sync.try_finish_frame_stage();
                            stage_sync.finish_post_stage();
                        }
                    })
                    .expect("couldn't spawn system thread");
            }

            // Main controller loop
            is_running.store(true, Ordering::SeqCst);
            while should_run.load(Ordering::SeqCst) {
                data_lock
                    .write_with(|data| {
                        let mut to_remove = data_to_remove.write().unwrap();
                        for tid in to_remove.drain(..) {
                            data.remove(&tid);
                        }

                        let mut to_add = data_to_add.write().unwrap();
                        for (tid, value) in to_add.drain(..) {
                            data.insert(tid, value);
                        }
                    })
                    .unwrap();

                events_lock
                    .write_with(|events| {
                        events.clear();
                        while let Ok(event) = event_receiver.try_recv() {
                            events.push(event);
                        }
                    })
                    .unwrap();
                
                let result = stage_controller.dispatch_frame();

                data_lock
                    .write()
                    .unwrap()
                    .insert(result.type_id(), Box::new(result));
            }
            is_running.store(false, Ordering::SeqCst);
        });

        info!("The engine has finished execution.");
    }
}
/// Top level controller held by the Engine Controller thread.
///
/// Manages dispatching systems' stages and their profiling.
pub struct TopLevelController {
    greenlight_barrier: Barrier,
    frame_barrier: Barrier,
    post_frame_barrier: Barrier,

    remaining_systems: Mutex<usize>,
    profiling_targets: Vec<Mutex<SystemProfiling>>,
}

impl<'a> TopLevelController {
    // ## Behavior of an internal system loop cycle:
    //  1. Wait for `greenlight_barrier` to fire off
    //     - This sync point allows the engine to setup processing of next
    //       frame
    //
    //  2. Enter the System `frame` callback:
    //     - Perform normal frame stuff
    //     - Wait for `frame_barrier` to fire off
    //     - Do any post frame stuff
    //  3. Wait for post_frame barrier to fire off
    //
    //  4. Go to step 1
    //

    pub fn new(system_count: usize) -> Self {
        Self {
            greenlight_barrier: Barrier::new(system_count + 1),
            frame_barrier: Barrier::new(system_count + 1),
            post_frame_barrier: Barrier::new(system_count + 1),

            remaining_systems: Mutex::new(system_count),
            profiling_targets: iter::repeat_with(Mutex::default)
                .take(system_count)
                .collect(),
        }
    }

    pub fn next_system(&'a self, name: String) -> SystemSync<'a> {
        let mut remaining = self.remaining_systems.lock().unwrap();
        assert_ne!(*remaining, 0, "Too many spawned systems");
        *remaining -= 1;

        SystemSync {
            name,
            frame_stage_start: None,
            frame_stage_finish: None,
            post_stage_start: None,
            post_stage_finish: None,
            profiling_target: &self.profiling_targets[*remaining],

            stage_point: 0,
            greenlight_barrier: &self.greenlight_barrier,
            frame_barrier: &self.frame_barrier,
            post_frame_barrier: &self.post_frame_barrier,
        }
    }

    pub fn dispatch_frame(&self) -> ProfilingResults {
        let start = Instant::now();
        self.greenlight_barrier.wait();
        self.frame_barrier.wait();
        self.post_frame_barrier.wait();
        let end = Instant::now();

        let total_time = end.duration_since(start);
        let fps = (1.0 / total_time.as_secs_f64()).floor() as u32;

        let systems = self
            .profiling_targets
            .iter()
            .map(|mutex| {
                let mut profiling = mutex.lock().unwrap();
                let result = profiling.clone();
                *profiling = SystemProfiling::default();
                result
            })
            .collect::<SmallVec<_>>();

        debug_assert!(!systems.spilled(), "too many systems, update profiling");

        ProfilingResults {
            fps,
            total_time,
            systems,
        }
    }
}

/// Manages synchronization and profiling of a system. Spawned by
/// [`SystemStageController`].
pub struct SystemSync<'a> {
    name: String,
    frame_stage_start: Option<Instant>,
    frame_stage_finish: Option<Instant>,
    post_stage_start: Option<Instant>,
    post_stage_finish: Option<Instant>,
    profiling_target: &'a Mutex<SystemProfiling>,

    stage_point: usize,
    greenlight_barrier: &'a Barrier,
    frame_barrier: &'a Barrier,
    post_frame_barrier: &'a Barrier,
}

impl<'a> SystemSync<'a> {
    pub const GREENLIGHT_STAGE: usize = 0;
    pub const FRAME_STAGE: usize = 1;
    pub const POST_FRAME_STAGE: usize = 2;
    pub const STAGE_COUNT: usize = 3;

    /// Awaits the frame's greenlight
    pub fn begin_frame(&mut self) {
        assert_eq!(
            self.stage_point,
            Self::GREENLIGHT_STAGE,
            "Frame wasn't yet finished"
        );
        self.greenlight_barrier.wait();
        self.stage_point += 1;
        self.frame_stage_start = Some(Instant::now());
    }

    pub fn finish_frame_stage(&mut self) {
        assert_eq!(self.stage_point, Self::FRAME_STAGE);
        self.frame_stage_finish = Some(Instant::now());
        self.frame_barrier.wait();
        self.stage_point += 1;
    }

    /// Returns true if frame barrier was awaited here
    pub fn try_finish_frame_stage(&mut self) -> bool {
        if self.stage_point == Self::FRAME_STAGE {
            self.finish_frame_stage();
            true
        } else {
            false
        }
    }

    pub fn begin_post_stage(&mut self) {
        assert_eq!(self.stage_point, Self::POST_FRAME_STAGE);
        self.post_stage_start = Some(Instant::now());
    }

    pub fn finish_post_stage(&mut self) {
        assert_eq!(self.stage_point, Self::POST_FRAME_STAGE);
        self.post_stage_finish = Some(Instant::now());

        self.profiling_target
            .with(|profiling| {
                let frame_start = self.frame_stage_start.take().unwrap();
                let frame_finish = self.frame_stage_finish.take().unwrap();
                let frame_stage_time = frame_finish.duration_since(frame_start);

                let post_start = self.post_stage_start.take();
                let post_finish = self.post_stage_finish.take();
                let post_stage_time = if let Some(post_start) = post_start {
                    Some(post_finish.unwrap().duration_since(post_start))
                } else {
                    None
                };

                *profiling = SystemProfiling {
                    name: self.name.clone(),
                    frame_stage_time,
                    post_stage_time,
                };
            })
            .unwrap();

        self.post_frame_barrier.wait();
        self.stage_point = 0;
    }

    pub fn stage_point(&self) -> usize {
        self.stage_point
    }
}

#[derive(Debug, Default, Clone)]
pub struct ProfilingResults {
    pub fps: u32,
    pub total_time: Duration,
    /// This Vec's capacity must be kept inline, if it spills, a debug assertion
    /// is raised
    pub systems: SmallVec<[SystemProfiling; 8]>,
}

#[derive(Debug, Default, Clone)]
pub struct SystemProfiling {
    pub name: String,
    pub frame_stage_time: Duration,
    pub post_stage_time: Option<Duration>,
}

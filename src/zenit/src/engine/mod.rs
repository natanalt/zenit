//! Engine core stuff

use self::{
    data::Data,
    system::{HasSystemInterface, System},
};
use crate::engine::system::SystemContext;
use log::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, self},
        Barrier, RwLock, Arc,
    },
    thread,
};
use winit::event::WindowEvent;

pub mod data;
pub mod system;

// TODO: use better hashing algorithms for HashMaps
//       A lot of HashMaps around here rely on integer keys, and since its usage
//       is internal to the engine, it doesn't need protection against any
//       attacks.

pub type TypeErasedSystemInterface = Box<dyn Any + Send + Sync>;
pub type SystemInterfaceTypeMap = HashMap<TypeId, TypeErasedSystemInterface>;
pub type TypeErasedData = Box<dyn Any + Send + Sync>;
pub type DataTypeMap = HashMap<TypeId, TypeErasedData>;

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
    /// Flag sighinfying whether the engine is still running. Accessible by
    /// the event loop.
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
        let events_lock = RwLock::new(Vec::with_capacity(32));

        // Lifetime of an internal system loop cycle:
        //  1. Frame beginning stage
        //     - Internally functions as a non-scriptable implementation detail
        //     - Engine controller spends this time collecting events, by asserting
        //       write control over the events list.
        //     - Systems internally flush out their data add/remove lists
        //  2. Frame stage
        //     - Programmable, normal frame processing
        //  3. Post-frame stage
        //     - Programmable, normal frame processing, but after a barrier,
        //       in case some systems need to do something here.

        // The +1s are accounting for the engine controller
        let begin_barrier = Barrier::new(systems.len() + 1);
        let frame_barrier = Barrier::new(systems.len());
        let post_frame_barrier = Barrier::new(systems.len() + 1);

        thread::scope(|scope| {
            for mut system in systems {
                trace!("Creating `{}` system instance", system.name());

                // Passing references into a `move` closure requires binding
                // them into variables. A bit annoying.
                let thread_references = (
                    &frame_barrier,
                    &post_frame_barrier,
                    &should_run,
                    &system_interfaces,
                    &begin_barrier,
                    &data_lock,
                    &events_lock,
                );

                thread::Builder::new()
                    .name(system.name().to_string())
                    .spawn_scoped(scope, move || {
                        let (
                            frame_barrier,
                            post_frame_barrier,
                            should_run,
                            system_interfaces,
                            begin_barrier,
                            data_lock,
                            events_lock,
                        ) = thread_references;

                        let mut ran_init = false;
                        let mut data_to_remove = vec![];
                        let mut data_to_add = vec![];

                        while should_run.load(Ordering::SeqCst) {
                            // Frame beginning stage
                            {
                                let mut data = data_lock.write().unwrap();

                                for to_remove in data_to_remove.drain(0..) {
                                    data.remove(&to_remove);
                                }

                                for (tid, value) in data_to_add.drain(0..) {
                                    data.insert(tid, value);
                                }

                                begin_barrier.wait();
                            }

                            // Frame & post-frame stages
                            let events = events_lock.read().unwrap();
                            let data = data_lock.read().unwrap();
                            let mut system_context = SystemContext {
                                should_run,
                                frame_barrier,
                                frame_barrier_waited: false,
                                system_interfaces,
                                data: &data,
                                data_to_remove: &mut data_to_remove,
                                data_to_add: &mut data_to_add,
                                events: &events,
                            };

                            if !ran_init {
                                system.init(&mut system_context);
                                ran_init = true;
                            }
                            system.frame(&mut system_context);

                            // Await any necessary barriers
                            if !system_context.frame_barrier_waited {
                                frame_barrier.wait();
                            }
                            post_frame_barrier.wait();
                        }
                    })
                    .expect("couldn't spawn system thread");
            }

            // TODO: move frame profiling somewhere else
            use std::time::{Duration, Instant};
            let mut _fps = 123;
            let mut frames_this_second = 0;
            let mut second_counter = Duration::ZERO;

            is_running.store(true, Ordering::SeqCst);
            while should_run.load(Ordering::SeqCst) {
                let frame_start = Instant::now();

                let mut events = events_lock.write().unwrap();
                events.clear();
                while let Ok(event) = event_receiver.try_recv() {
                    events.push(event);
                }
                drop(events);

                // TODO: add freeze/deadlock checks
                begin_barrier.wait();
                post_frame_barrier.wait();

                let frame_end = Instant::now();
                let frame_time = frame_end.duration_since(frame_start);

                frames_this_second += 1;
                second_counter += frame_time;
                if second_counter > Duration::from_secs(1) {
                    second_counter = Duration::ZERO;
                    _fps = frames_this_second;
                    frames_this_second = 0;
                }
            }
            is_running.store(false, Ordering::SeqCst);
        });

        info!("The engine has finished execution.");
    }
}

/// Communication between the main engine control thread and the main GUI event
/// loop thread. Basically it's the way the GUI thread can:
///  * Report new window and system events to the engine.
///  * Report window closing done by the user, requiring immediate shutdown.
///    Note, that when the window close button is pressed, the window is closed
///    instantly.
pub struct EngineCommunication {
    /// Send channel for event loop's events
    pub event_sender: Sender<WindowEvent<'static>>,
    /// If true, next iteration of the game loop will process a new frame
    pub should_run: Arc<AtomicBool>,
    /// If true, the engine is still running.
    pub is_running: Arc<AtomicBool>,
}

/// It builds the engine. Very surprising, I know
#[derive(Default)]
pub struct EngineBuilder {
    systems: Vec<Box<dyn for<'a> System<'a>>>,
    system_interfaces: SystemInterfaceTypeMap,
    data: DataTypeMap,
}

impl EngineBuilder {
    /// Creates and includes a [`System`] instance, if it implements [`Default`]
    pub fn make_system<S>(mut self) -> Self
    where
        S: for<'a> System<'a> + HasSystemInterface + Default,
    {
        let system = Box::new(S::default());
        let system_interface = Box::new(system.create_system_interface());
        self.systems.push(system);
        self.system_interfaces
            .insert(TypeId::of::<S::SystemInterface>(), system_interface);
        self
    }

    /// Includes specified [`Data`]
    pub fn with_data<D>(mut self, data: D) -> Self
    where
        D: Data,
    {
        self.data.insert(TypeId::of::<D>(), Box::new(data));
        self
    }

    /// Creates and includes a [`Data`] instance, if it implements [`Default`]
    pub fn make_data<D>(self) -> Self
    where
        D: Data + Default,
    {
        self.with_data(D::default())
    }

    /// Finalizes the engine build
    pub fn build(self) -> (Engine, EngineCommunication) {
        let (event_sender, event_receiver) = mpsc::channel();
        let should_run = Arc::new(AtomicBool::new(true));
        let is_running = Arc::new(AtomicBool::new(false));

        let engine = Engine {
            systems: self.systems,
            system_interfaces: self.system_interfaces,
            data: self.data,
            event_receiver,
            should_run: should_run.clone(),
            is_running: is_running.clone(),
        };

        let comms = EngineCommunication {
            event_sender,
            should_run,
            is_running,
        };

        (engine, comms)
    }
}

//! Engine core stuff

use crate::engine::system::SystemContext;

use self::{
    data::Data,
    system::{HasSystemInterface, System},
};
use log::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Barrier, RwLock,
    },
    thread,
};

pub mod data;
pub mod system;

pub type TypeErasedSystemInterface = Box<dyn Any + Sync>;
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
        } = self;

        let should_run = AtomicBool::new(false);

        let data = RwLock::new(data);

        // A hidden barrier working more as an implementation detail
        // Allows for runner synchronization and updating of fields like data
        // based on system outputs
        let begin_barrier = Barrier::new(systems.len());

        // Behavior and usage of these barriers is documented somewhere in system.rs
        let frame_barrier = Barrier::new(systems.len());
        let post_frame_barrier = Barrier::new(systems.len());

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
                    &data,
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
                        ) = thread_references;

                        let mut ran_init = false;
                        let mut data_to_remove = vec![];
                        let mut data_to_add = vec![];

                        while should_run.load(Ordering::SeqCst) {
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

                            let data = data_lock.read().unwrap();
                            let system_context = SystemContext {
                                frame_barrier,
                                post_frame_barrier,
                                should_run,
                                system_interfaces,
                                data: &data,
                                data_to_remove: &mut data_to_remove,
                                data_to_add: &mut data_to_add,
                            };

                            if !ran_init {
                                system.init(&system_context);
                                ran_init = true;
                            }
                            system.frame(&system_context);
                        }
                    })
                    .expect("couldn't spawn system thread");
            }

            should_run.store(true, Ordering::SeqCst);
        });

        info!("All systems have finished execution.");
    }
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
    pub fn build(self) -> Engine {
        Engine {
            systems: self.systems,
            system_interfaces: self.system_interfaces,
            data: self.data,
        }
    }
}

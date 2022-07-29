//! Engine core stuff

use self::{
    data::Data,
    system::{HasSystemInterface, System, SystemContext},
};
use log::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    thread::{self, ScopedJoinHandle},
};

pub mod data;
pub mod system;

/// Main engine controller.
pub struct Engine {
    systems: Vec<(Box<dyn System>, Box<dyn Any>)>,
}

impl Engine {
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    /// Runs the engine in the current thread, until it exits.
    pub fn run(self) {
        let Self { systems } = self;

        /// System state local to [`Engine::run`]
        struct SystemInstance<'scope> {
            /// See [`system::HasSystemInterface`] for details.
            system_interface: Box<dyn Any>,
            /// Join handle of the engine-controlled system thread.
            thread_handle: ScopedJoinHandle<'scope, ()>,
        }

        let should_run = AtomicBool::new(true);

        let frame_barrier = Barrier::new(systems.len());
        let post_frame_barrier = Barrier::new(systems.len());

        thread::scope(|scope| {
            let mut instances = HashMap::with_capacity(systems.len());
            for (mut system, system_interface) in systems {
                trace!("Creating `{}` system instance", system.name());

                let system_context = SystemContext {
                    frame_barrier: &frame_barrier,
                    post_frame_barrier: &post_frame_barrier,
                    should_run: &should_run,
                };

                let tid = (&*system).type_id();
                let instance = SystemInstance {
                    system_interface,
                    thread_handle: thread::Builder::new()
                        .name(system.name().to_string())
                        .spawn_scoped(scope, move || {
                            while system_context.should_run.load(Ordering::SeqCst) {
                                system.frame(system_context.clone());
                            }
                        })
                        .expect("couldn't spawn system thread"),
                };

                instances.insert(tid, instance);
            }
        });

        info!("All systems have finished execution.");
    }
}

/// It builds the engine. Very surprising, I know
#[derive(Default)]
pub struct EngineBuilder {
    systems: Vec<(Box<dyn System>, Box<dyn Any>)>,
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl EngineBuilder {
    /// Creates and includes a [`System`] instance, if it implements [`Default`]
    pub fn make_system<S>(mut self) -> Self
    where
        S: System + HasSystemInterface + Default,
    {
        let system = Box::new(S::default());
        let system_interface = Box::new(system.create_system_interface());
        self.systems.push((system, system_interface));
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
        }
    }
}

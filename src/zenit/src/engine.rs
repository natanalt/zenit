//! # Engine core stuff

use std::{any::{Any, TypeId}, sync::{Barrier, Arc}, collections::HashMap};
use log::*;

/// Main engine controller.
pub struct Engine {
    systems: Vec<Box<dyn System>>,
}

impl Engine {
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    pub fn start(self) -> Self {
        let context = Arc::new(SystemContext {
            frame_barrier: Barrier::new(self.systems.len()),
            post_frame_barrier: Barrier::new(self.systems.len()),
        });

        for system in &self.systems {
            trace!("Starting system `{}`...", system.name());

            assert!(!system.is_running());
            system.start_thread(context.clone());
        }

        self
    }
}

#[derive(Default)]
pub struct EngineBuilder {
    systems: Vec<Box<dyn System>>,
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl EngineBuilder {
    pub fn make_system<S>(mut self) -> Self
    where
        S: System + Default
    {
        self.systems.push(Box::new(S::default()));
        self
    }

    pub fn with_data<D>(mut self, data: D) -> Self
    where
        D: Data
    {
        self.data.insert(TypeId::of::<D>(), Box::new(data));
        self
    }

    pub fn make_data<D>(self) -> Self
    where
        D: Data + Default
    {
        self.with_data(D::default())
    }

    pub fn build(self) -> Engine {
        Engine {
            systems: self.systems,
        }
    }
}

/// Represents a piece of globally readable data
pub trait Data: Any + Send + Sync {
    /// What kind of data is stored by this container?
    type Storage;

    /// Reads the current value of this data
    fn read(&self) -> Self::Storage;    
}

/// Extends [`Data`] by adding atomic write support
pub trait WritableData: Data {
    /// Writes a new value to this data. Calls the provided closure
    /// for the duration of the write, while locking the value.
    /// 
    /// Parameter in provided closure is the old value.
    fn write_with<F>(&self, f: F)
    where
        F: FnOnce(Self::Storage) -> Self::Storage;

    /// Writes a new value to this data
    fn write(&self, value: Self::Storage) {
        self.write_with(move |_| value);
    }
}

/// Implements an engine system.
/// 
/// A system is a module which implements a certain piece of functionality,
/// like rendering, physics, or the overall entity/scene system. Each system
/// runs in a dedicated thread.
/// 
/// Systems perform their work each frame and then wait for other systems
/// to finish whatever they were doing using a provided [`Barrier`]
pub trait System: Any + Sync {
    /// Returns this system's name
    fn name(&self) -> &str;
    /// Starts the system's worker thread.
    fn start_thread(&self, context: Arc<SystemContext>);
    /// Requests the system's worker thread to be stopped.
    fn request_stop(&self);
    /// Waits until this system's worker thread stops. Returns immediately if
    /// it's not already running.
    fn wait_for_stop(&self);
    /// Checks whether the system's worker thread is running at the moment.
    fn is_running(&self) -> bool;
}

/// Context passed to each system thread, which allows for communication with
/// the engine.
/// 
/// ## Sync barriers
/// The context includes 2 barriers: frame barrier and post frame barrier.
/// They are meant to be awaited in a very specific order:
/// ```text
///               ↓ frame_barrier    ↓ post_frame_barrier
/// ┌─────────────┬──────────────────┐
/// │ Frame stuff │ Post frame stuff │
/// └─────────────┴──────────────────┘
/// ↑ frame start                    ↑ frame end
/// ```
/// 
/// Systems that don't make use of post frame period can finish their frames
/// like this:
/// ```ignore
/// fn system_thread(context: Arc<SystemContext>) {
///     loop {
///         do_some();
///         frame_stuff();
///         over_here();
///         
///         // We're done, let's wait for other systems
///         context.frame_barrier.wait();
///         // other systems may do some things here, we don't care
///         context.post_frame_barrier.wait();
///     }
/// }
/// ``` 
pub struct SystemContext {
    /// Barrier that synchronizes every frame's basic end, starting the post
    /// frame phase, allowing systems to do whatever late operations they need
    /// to do after primary processing.
    pub frame_barrier: Barrier,

    /// Barrier that synchronizes the post frame phase. After this barrier
    /// resets, a new frame will begin to process.
    pub post_frame_barrier: Barrier,
}
